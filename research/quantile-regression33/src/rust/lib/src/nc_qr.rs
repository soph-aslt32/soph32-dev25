//! # About
//! 
//! Non Crossing Quantile Regressionに使用されるニューラルネットワークの実装です．
//! 
//! ## Detail
//! 
//! 入力される`Tensor`の形状は，`(batch_size, n_features)`を想定しています．
//! 
//! 出力される`Tensor`の形状は，`(batch_size, N, n_quantiles)`を想定しています．
//! 
//! ここで，`N`は強化学習における現在状態における行動の数を表します．
//! 

mod net;

pub use net::Net;
