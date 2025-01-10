use anyhow::Result;
use tch::nn::{LinearConfig, ModuleT, SequentialT};
use tch::{nn, Device, Tensor, Kind};

pub struct NetConfig {
    /// Embeddingネットワークの中間層のユニット数です．
    ///
    /// 例えば，`[64, 64]`とすると，(input_dim, 64) -> (64, 64)のネットワークが生成されます．
    ///
    /// 最後の要素の値が`embedding_dim`になります．
    embedding_net_units: Vec<i64>,

    /// 入力データの次元数です．
    input_dim: i64,

    /// 行動の次元数です．
    action_dim: i64,

    /// 出力データの分位点数です．
    n_quantile: i64,
}

/// # About
///
/// Non Crossing Quantile Regressionに使用されるニューラルネットワーク本体です．
///
/// ## Detail
///
/// ニューラルネットワークの形状の説明はドキュメント(nc_qr.drawio)を参照してください．
///
/// 中間層の活性化関数はMishが使用されます．
pub struct Net {
    /// 訓練モードのフラグです．
    train: bool,

    /// Variable Storeです．
    vs: nn::VarStore,

    /// Embeddingネットワークです．
    embedding_net: SequentialT,

    /// αネットワークです．
    alpha_net: SequentialT,

    /// βネットワークです．
    beta_net: SequentialT,

    /// non-crossing制約を満たすためのネットワークです．
    nc_net: SequentialT,

    /// 設定値です．
    config: NetConfig,
}

impl Net {
    pub fn new(
        config: NetConfig,
        device: Device,
    ) -> Self {
        let train = true;

        let input_dim = config.input_dim;
        let action_dim = config.action_dim;
        let n_quantile = config.n_quantile;

        // Variable Storeを生成.
        let vs = nn::VarStore::new(device);
        let p = &vs.root();

        // Embeddingネットワークを生成.
        let embedding_net = Self::mlp_mish(
            input_dim,
            config.embedding_net_units.clone(),
            &(p / "embedding_net"),
        );

        // embedding_netの出力次元数.
        let embedding_dim = config.embedding_net_units.last().unwrap().clone();

        // αネットワークを生成.
        let alpha_net = {
            let mut seq_t = nn::seq_t();
            let c = LinearConfig::default();
            seq_t = seq_t.add(nn::linear(&(p / "alpha_net"), embedding_dim, 1, c));
            seq_t = seq_t.add_fn(|xs| xs.relu());
            seq_t
        };

        // βネットワークを生成.
        let beta_net = {
            let mut seq_t = nn::seq_t();
            let c = LinearConfig::default();
            seq_t = seq_t.add(nn::linear(&(p / "beta_net"), embedding_dim, 1, c));
            seq_t
        };

        // non-crossing制約を満たすためのネットワークを生成.
        let nc_net = {
            let mut seq_t = nn::seq_t();
            let c = LinearConfig::default();
            let action_dim = action_dim;
            let n_quantile = n_quantile;
            seq_t = seq_t.add(nn::linear(
                &(p / "nc_net"),
                embedding_dim,
                action_dim * n_quantile,
                c,
            ));
            // softmax適用のためのTensorの形状変換.
            seq_t = seq_t.add_fn(move |xs| xs.contiguous().view([-1, action_dim, n_quantile]));
            // softmax適用
            seq_t = seq_t.add_fn(|xs| xs.softmax(-1, xs.kind()));
            // softmax適用後のTensorを元に戻す処理．
            seq_t = seq_t.add_fn(move |xs| xs.contiguous().view([-1, action_dim * n_quantile]));
            seq_t
        };

        Self {
            train,
            vs,
            embedding_net,
            alpha_net,
            beta_net,
            nc_net,
            config,
        }
    }

    /// # About
    ///
    /// ニューラルネットワークの順伝播処理を行います．
    ///
    /// ## Args
    ///
    /// * `x` - 入力データです．(batch_size, input_dim)の形状の`Tensor`です．
    ///
    /// ## Returns
    ///
    /// * `Tensor` - 出力データです．(batch_size, action_dim, n_quantile)の形状の`Tensor`です．
    pub fn forward(&self, x: &Tensor) -> Tensor {
        let embedding = self.embedding_net.forward_t(x, self.train);

        // α, βを計算. (batch_size, action_dim)
        let alpha = self.alpha_net.forward_t(&embedding, self.train);
        let beta = self.beta_net.forward_t(&embedding, self.train);

        // non-crossing制約を満たすためのネットワークを適用. (batch_size, action_dim * n_quantile)
        let nc = self.nc_net.forward_t(&embedding, self.train);
        // Tensorの最後の次元で累積和を計算．(batch_size, action_dim * n_quantile)
        let nc = nc.contiguous().view([nc.size()[0], self.config.action_dim, self.config.n_quantile]);
        let nc = nc.cumsum(-1, nc.kind());
        let nc = nc.contiguous().view([nc.size()[0], -1]);

        // alpha * nc + betaを計算するためにTensorの形状を変換．(batch_size, action_dim, 1)
        let alpha = alpha.unsqueeze(-1);
        let beta = beta.unsqueeze(-1);
        // ncの形状を(batch_size, action_dim, n_quantile)に変換．
        let nc = nc.contiguous().view([nc.size()[0], self.config.action_dim, self.config.n_quantile]);

        // alpha * nc + betaを計算．(batch_size, action_dim, n_quantile)
        let y = alpha * nc + beta;

        // 出力データの形状を(batch_size, action_dim * n_quantile)に変換．
        y.contiguous().view([y.size()[0], -1])
    }

    pub fn set_train(&mut self, train: bool) {
        self.train = train;
    }

    fn mlp_mish(input_dim: i64, units: Vec<i64>, vs: &nn::Path) -> SequentialT {
        // 初期値.
        let last_ix = units.len() - 1;
        let mut in_dim = input_dim;
        let mut seq_t = nn::seq_t();

        // ユニット数分の層を生成.
        for (ix, unit) in units.iter().enumerate() {
            let p = vs / format!("fc_{}", ix);
            let c = LinearConfig::default();
            seq_t = seq_t.add(nn::linear(p, in_dim, *unit, c));

            // 最後の層以外はMishを適用
            if ix != last_ix {
                seq_t = seq_t.add_fn(|xs| xs.mish());
            }
            // 最後の層はReLUを適用
            else {
                seq_t = seq_t.add_fn(|xs| xs.relu());
            }

            in_dim = *unit;
        }

        seq_t
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_net_new_forward() -> Result<()> {
        let device = Device::Cpu;
        let config = NetConfig {
            embedding_net_units: vec![8, 8],
            input_dim: 4,
            action_dim: 2,
            n_quantile: 3,
        };
        let net = Net::new(config, device);

        // (batch_size = 2, input_dim = 4)
        let x = Tensor::randn(&[2, 4], (Kind::Float, device));
        x.print();
        // (batch_size = 2, action_dim = 2, n_quantile = 3) => (2, 6)
        let y = net.forward(&x);
        y.print();

        Ok(())
    }

    #[test]
    fn test_tensor_unsqueeze() -> Result<()> {
        let device = Device::Cpu;
        let x = Tensor::randn(&[2, 2], (Kind::Float, device));
        x.print();
        let y = x.unsqueeze(-1);
        y.print();

        Ok(())
    }

    #[test]
    fn test_tensor_a_times_nc() -> Result<()> {
        let device = Device::Cpu;
        let a = Tensor::randint(10, &[2, 6], (Kind::Float, device));
        a.print();
        let b = Tensor::randint(10, &[2, 2], (Kind::Float, device));
        b.print();

        let a = a.view([-1, 2, 3]);
        a.print();
        let b = b.view([-1, 2, 1]);
        b.print();

        let c = a * b;
        c.print();

        Ok(())
    }

    #[test]
    fn test_softmax() -> Result<()> {
        let device = Device::Cpu;
        let x = Tensor::randn(&[4, 2, 3], (Kind::Float, device));
        x.print();
        let y = x.softmax(-1, x.kind());
        y.print();

        Ok(())
    }

    #[test]
    fn test_tensor_view_softmax_view_test() -> Result<()> {
        let device = Device::Cpu;
        let x = Tensor::randn(&[4, 6], (Kind::Float, device));
        x.print();
        let y = x.view([-1, 2, 3]);
        y.print();
        let z = y.softmax(-1, y.kind());
        z.print();
        let w = z.view([-1, 6]);
        w.print();

        Ok(())
    }

    #[test]
    fn test_tensor_view_cumsum_view_test() -> Result<()> {
        let device = Device::Cpu;
        let x = Tensor::randn(&[2, 6], (Kind::Float, device));
        x.print();
        let y = x.view([-1, 2, 3]);
        y.print();
        let z = y.cumsum(-1, y.kind());
        z.print();
        let w = z.view([-1, 6]);
        w.print();

        Ok(())
    }
}
