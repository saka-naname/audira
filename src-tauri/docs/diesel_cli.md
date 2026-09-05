# Diesel CLIを使用したスキーマ編集方法

AudiraのSQLiteスキーマは、`mise`が管理するDiesel CLIで変更します。個人環境へ別途インストールした`diesel`コマンドは使いません。これにより、開発者間でCLIのバージョンと有効なデータベース機能を揃えます。

このガイドは、初めて作業するときに「初回セットアップ」から「実DBへ適用する」までを順に読んでください。2回目以降は、目的に合う見出しだけ参照できます。

## 初回セットアップ

リポジトリルートで次のコマンドを実行します。

```sh
mise install
mise exec -- diesel --version
```

`mise.toml`で固定した、SQLite対応のDiesel CLIが使われます。`cargo install diesel_cli`や`diesel setup`を個別に実行する必要はありません。プロジェクトの設定は[`src-tauri/diesel.toml`](../diesel.toml)にあり、migrationディレクトリと`schema.rs`の生成先も設定済みです。

## マイグレーション

新しいマイグレーションを作成する際は、プロジェクトルートから下を実行します。マイグレーション名は、変更内容が分かるsnake_caseにします。

```sh
mise exec -- diesel \
  --config-file src-tauri/diesel.toml \
  migration generate add_library_roots
```

実行すると、`src-tauri/diesel_migrations_transitive`の下にタイムスタンプ付きのディレクトリが作られます。

```text
src-tauri/diesel_migrations_transitive/
└── <timestamp>_add_library_roots/
    ├── up.sql
    └── down.sql
```

`src-tauri`へ移動して作業する場合は、設定ファイルを省略できます。

```sh
cd src-tauri
mise exec -- diesel migration generate add_library_roots
```

すでに適用されたmigrationは編集しません。修正が必要になった場合は、新しいmigrationを追加してください。

> [!IMPORTANT]
> 
> 現在 sqlx から diesel への移行作業を実行中のため、diesel 側のマイグレーションは正式なものではありません。
> [#38](https://github.com/saka-naname/audira/issues/38) で再度構築を行う予定です。

## 実DBへ適用する

最初に一時DBへ適用します。次の例はmacOSとLinuxを想定しています。

```sh
tmp_dir=$(mktemp -d)
test_db="$tmp_dir/audira.db"

mise exec -- diesel \
  --config-file src-tauri/diesel.toml \
  migration run \
  --database-url "$test_db"

mise exec -- diesel \
  --config-file src-tauri/diesel.toml \
  migration redo \
  --database-url "$test_db"
```

テーブル再作成やデータ変換を含む場合は、テストデータを投入し、`redo`の前後で値と外部キーを確認します。

```sh
sqlite3 "$test_db" "PRAGMA foreign_key_check;"
```

検証が済んだら、Audiraを終了した状態で開発用の実DBへ適用します。

```sh
mise run diesel:migrate
```

このタスクは`mise run database-path`でAudiraのDBパスを取得します。`diesel:redo`も同じ実DBを対象にするため、通常の確認には使わないでください。実DBで戻す必要がある場合は、先にDBファイルをバックアップし、migrationのデータ損失条件を確認します。

## `schema.rs`の確認

> [!WARNING]
> 
> `schema.rs` は手動で編集しないでください。

`migration run`が成功すると、`diesel.toml`の設定により[`src/schema.rs`](../src/schema.rs)が更新されます。手動で再生成したい場合は次を実行します。

```sh
mise run diesel:schema
```

生成後はRust側の検査を実行します。

```sh
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
git diff --check
```

`schema.rs`に意図しない型やテーブルが現れた場合は、生成結果を直接直さず、migrationまたは`diesel.toml`を修正して再生成します。

## 目的別コマンド

| 目的 | コマンド |
| --- | --- |
| CLIをインストールする | `mise install` |
| migrationを生成する | `mise exec -- diesel --config-file src-tauri/diesel.toml migration generate <name>` |
| 実DBへ未適用migrationを適用する | `mise run diesel:migrate` |
| 実DBのスキーマから`schema.rs`を再生成する | `mise run diesel:schema` |
| 実DBで最後のmigrationを戻して再適用する | `mise run diesel:redo` |
| AudiraのDBパスを確認する | `mise run database-path` |

`diesel:redo`は実DBを書き換えます。migrationの往復確認には、一時DBと明示的な`--database-url`を使ってください。

## 用語

- **migration**: DBスキーマや既存データを、バージョン順に変更する処理です。
- **実DB**: `mise run database-path`が返す、Audiraが通常使用するSQLiteファイルです。
- **一時DB**: migrationの動作確認のために作り、確認後に破棄するSQLiteファイルです。
- **`schema.rs`**: DieselがDBスキーマから生成するRustコードです。手作業では変更しません。
- **DDL**: `CREATE TABLE`や`ALTER TABLE`など、DBの構造を変更するSQLです。

最終更新: 2026-09-05
