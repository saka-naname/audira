# AGENTS.md

このリポジトリは Audira です。ローカル上の音楽ファイルを管理するための Tauri アプリケーションで、フロントエンドは React + TypeScript + TanStack Router、バックエンドは Rust + Tauri + SQLite(sqlx) で構成されています。

## 開発前に確認すること

- 既存の変更を勝手に戻さないでください。作業前後に `git status --short` を確認し、無関係な差分は触らないでください。
- 生成ファイルは原則として直接編集しないでください。
  - `src/routeTree.gen.ts`
  - `src-tauri/src/models/*.rs`
- フロントエンドの共通 UI は `src/components/ui` にあります。このディレクトリは Biome の対象外で、shadcn 由来のコンポーネントとして扱います。
- パスエイリアスは `@/*` を使います。例: `@/features/library/components/library-sidebar`

## よく使うコマンド

- フロントエンド開発: `npm run dev`
- Tauri 開発: `npm run tauri dev`
- フロントエンドビルド: `npm run build`
- フロントエンド lint: `npm run lint`
- フロントエンド format: `npm run format`
- フロントエンド総合チェック: `npm run check`
- Rust チェック: `cd src-tauri && cargo check`
- Rust テスト: `cd src-tauri && cargo test`
- Rust format: `cd src-tauri && cargo fmt`

## ツールとフォーマット

- Node と Rust のバージョンは `mise.toml` を参照してください。
  - Node: `24.15.0`
  - Rust: `1.95.0`
- TypeScript は `strict` です。未使用 import / 変数 / 引数を残さないでください。
- Biome の設定は `biome.json` にあります。
  - インデントはスペース 2。
  - 文字列は double quote。
  - Tailwind class は `cn`, `twMerge`, `twJoin` 内も含めてソート対象です。
- Rust は `cargo fmt` の標準フォーマットに従います。

## フロントエンド構成

主なディレクトリ:

- `src/main.tsx`: React と TanStack Router のエントリポイント。
- `src/routes`: TanStack Router のファイルベースルーティング。
- `src/features`: 機能単位の実装。
- `src/components/ui`: shadcn / 共通 UI プリミティブ。
- `src/hooks`: アプリ全体で共有する汎用 hook。
- `src/lib`: 汎用ユーティリティ。現状は `cn` など。
- `src/styles.css`: Tailwind v4、theme token、global style。

### ルーティング規則

- ルートは TanStack Router の file based routing で追加してください。
- `src/routes` では route 定義とページ/レイアウトの組み立てを中心にし、重い UI や状態管理は `src/features/<feature>` に寄せます。
- layout route は `route.tsx`、index route は `index.tsx` の既存パターンに合わせます。
- `src/routeTree.gen.ts` は TanStack Router による生成物です。直接編集しないでください。
- 画面遷移は `Link` または `useNavigate` を使い、手動で URL 文字列を組み立てないでください。

### feature 構成規則

新しい機能は `src/features/<feature-name>` にまとめます。今後は以下の形を標準にします。

```text
src/features/<feature-name>/
  components/
  hooks/
  api/
  types.ts
  utils.ts
```

- `components`: feature 固有の UI コンポーネント。
- `hooks`: feature 固有の state、TanStack Query、Tauri invoke 呼び出しの組み立て。
- `api`: Tauri command 呼び出しなど、外部境界に近い処理。
- `types.ts`: feature 内で共有する型。
- `utils.ts`: feature 内だけで使う純粋関数。

小さい feature では未使用ディレクトリを無理に作らなくて構いません。ただし、成長したら上記に分割してください。

### コンポーネント規則

- React コンポーネントは関数コンポーネントで書きます。
- feature 固有のコンポーネントは `src/features/<feature>/components` に置きます。
- 複数 feature で使う UI プリミティブだけ `src/components/ui` または `src/components` に昇格します。
- 大規模なコンポーネントは `components` 配下に専用ディレクトリを作り、`index.tsx` を外部公開用のエントリーポイントにしてください。
- 大規模コンポーネントの内部 UI は同じディレクトリ内の小さなコンポーネントに分割し、外部からは原則として `index.tsx` だけを import します。
- 例:

```text
src/features/<feature>/components/big-component/
  index.tsx
  component-a.tsx
  component-b.tsx
```

```tsx
import BigComponent from "@/features/<feature>/components/big-component";
```

- `index.tsx` は外部 API を安定させる場所です。内部コンポーネント、内部 hook、表示用 helper をむやみに再 export しないでください。
- アイコンは既存に合わせて `@tabler/icons-react` を使います。
- className の結合は `@/lib/utils` の `cn` を使います。
- Tailwind class は既存の theme token、shadcn token、CSS variables を優先してください。
- UI テキストは現状に合わせて日本語を基本にします。

### Tauri command 呼び出し

- フロントエンドから Rust への呼び出しは `@tauri-apps/api/core` の `invoke` を使います。
- invoke 名と引数名は Rust 側の command に合わせます。例: `invoke("scan_library", { baseDir })`
- command 呼び出しを複数箇所で使う場合は、route や component に直書きせず `src/features/<feature>/api` または `hooks` に切り出してください。

## バックエンド構成

主なディレクトリ:

- `src-tauri/src/lib.rs`: Tauri Builder、plugin、DB 初期化、command 登録。
- `src-tauri/src/commands`: Tauri command の境界層。
- `src-tauri/src/repository`: DB アクセス層。
- `src-tauri/src/models`: sqlx-gen による DB モデル。
- `src-tauri/src/constants.rs`: 共有定数。
- `src-tauri/migrations`: SQLite migration。
- `src-tauri/docs/generate_models.md`: sqlx-gen によるモデル生成メモ。

## バックエンドの3層アーキテクチャ

今後の Tauri command は、原則として次の3層に分けて実装してください。

```text
commands -> services -> repository
```

### 1. commands 層

配置: `src-tauri/src/commands/<feature>.rs`

- `#[tauri::command]` を置く外部境界です。
- 引数の受け取り、Tauri state の取り出し、ユーザー向けエラー文字列への変換を担当します。
- ビジネスロジック、DB クエリ、ファイル走査などを厚く書かないでください。
- 返り値はフロントエンドに返す DTO または `Result<T, String>` を基本にします。

### 2. services 層

配置: `src-tauri/src/services/<feature>_service.rs`

- 今後追加する標準層です。現状の `scan_library` にはファイル走査、ハッシュ計算、メタデータ抽出、アルバム関連付けが command 内にありますが、新規実装や大きな改修では service へ移してください。
- ユースケース単位の処理、トランザクション制御、repository の組み合わせ、ファイルシステムや `lofty` を使ったドメイン処理を担当します。
- Tauri の `AppHandle` や `Window` に依存しなくてよい処理は、service 以下へ逃がしてください。
- service のエラーは内部向け enum または `anyhow` 相当の型を検討し、command 境界でユーザー向け文言に変換します。現状の依存にない crate を追加する場合は、既存方針に合うか確認してください。

### 3. repository 層

配置: `src-tauri/src/repository/<table>_repository.rs`

- SQLite への読み書きだけを担当します。
- `SqliteConnection` または必要に応じて `SqlitePool` を受け取り、SQL と `sqlx` の bind/fetch を閉じ込めます。
- repository では Tauri API、ファイルシステム走査、音声メタデータ解析を扱わないでください。
- insert/update 用の params struct は repository の近くに置きます。例: `InsertSongsParams`
- SQL は migration のテーブル定義と `src-tauri/src/models` の型に合わせます。

### モデルと migration

- DB スキーマ変更は `src-tauri/migrations` に migration を追加してください。
- `src-tauri/src/models/*.rs` は `sqlx-gen` 生成物です。直接編集せず、必要なら migration 適用後に再生成します。
- 生成手順は `src-tauri/docs/generate_models.md` を参照してください。
- DB の仕様説明を更新する場合は `docs/schema.md` も合わせて更新してください。

## 命名規則

- TypeScript/React ファイルは kebab-case を基本にします。例: `library-sidebar.tsx`
- React コンポーネント名は PascalCase。例: `LibrarySidebar`
- hook は `useXxx`。
- Rust module / file は snake_case。
- Rust struct / enum は PascalCase、関数は snake_case。
- DB のカラム名は snake_case。フロントエンドに返す型で camelCase が必要な場合は DTO を用意してください。

## 実装時の注意

- 共有化は早すぎないようにし、まずは feature 内に置いてください。複数 feature から自然に使われるようになったら共通化します。
- 長い処理は UI をブロックしない設計にしてください。ライブラリスキャンのような処理は progress / cancel / logging の拡張を考慮します。
- ファイルパスや音楽メタデータは欠損・文字化け・未知形式があり得ます。`unwrap` / `expect` はアプリ起動時の致命的初期化以外では避け、ユーザー向けエラーに変換してください。
- Tauri command を追加したら `src-tauri/src/lib.rs` の `tauri::generate_handler!` に登録してください。
- Tauri plugin の権限が必要な場合は `src-tauri/capabilities/default.json` も確認してください。
- 大量ファイル処理、DB トランザクション、重複判定、アルバム関連付けは regression が起きやすいので、Rust 側の単体テストまたは小さな統合テストを優先して追加してください。

## 作業完了前の確認

- フロントエンドのみの変更: `npm run check` と必要に応じて `npm run build`
- Rust のみの変更: `cd src-tauri && cargo fmt && cargo check`
- Tauri command やフロント/バックエンド境界の変更: `npm run build` と `cd src-tauri && cargo check`
- UI 変更: ローカルで画面を開き、主要 viewport で崩れやテキストの重なりがないことを確認してください。
