※ Deprecated / sqlx から diesel への移行のため

一旦。後々タスクランナーを用意したい

```zsh
sqlx-gen generate entities -u sqlite://path-to.db -x _sqlx_migrations -o src-tauri/src/models
```
