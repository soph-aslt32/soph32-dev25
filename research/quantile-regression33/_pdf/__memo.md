# 論文memo

Exploration in Deep Reinforcement Learning: From Single-Agent to Multiagent Domain
<https://arxiv.org/abs/2109.06668>
探索手法をまとめたサーベイ論文．

```txt
深層強化学習（Deep Reinforcement Learning: DRL）と深層マルチエージェント強化学習（Deep Multi-agent Reinforcement Learning: MARL）は、ゲームAI、自律走行車、ロボット工学など、幅広い領域で大きな成功を収めている。しかし、DRLやディープMARLエージェントは、比較的単純な問題設定であっても数百万回のインタラクションを必要とする非効率なサンプルであることが広く知られており、実業界への幅広い応用や展開が妨げられています。すなわち、いかに効率的に環境を探索し、最適なものに向けた政策学習に役立つ有益な経験を収集するかという問題である。
この問題は、報酬が疎であり、ノイズの多い注意散漫な環境、長い水平時間、非定常な共同学習者を含む複雑な環境では、より困難となる。本稿では、シングルエージェントとマルチエージェントのRLにおける既存の探索手法について包括的なサーベイを行う。まず、効率的な探索に対するいくつかの重要な課題を明らかにする。次に、不確実性志向の探索と内発的動機づけ志向の探索という2つの主要なカテゴリーに分類することで、既存のアプローチの体系的なサーベイを提供する。上記2つの主要な枝葉の他にも、異なるアイデアや技術を持つ注目すべき探索手法を含む。アルゴリズム分析に加え、一般的に使用されるベンチマークを用いた、DRLのための様々な探索手法の包括的かつ統一的な実証的比較も行います。アルゴリズムと実証的な調査により、DRLとdeep MARLにおける探索の未解決の問題をまとめ、今後の方向性を示します。
```
