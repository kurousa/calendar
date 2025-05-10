# カレンダーCLIアプリケーション

このプロジェクトは、Rustで実装されたコマンドラインインターフェース（CLI）のカレンダーアプリケーションです。予定の追加、削除、一覧表示ができ、重複する予定の登録を防止する機能を持っています。

## 機能

- スケジュールの一覧表示
- 新しい予定の追加（重複チェック機能付き）
- 予定の削除
- JSONファイルを使用したデータの永続化

## 動作環境

- Rust 2021 Edition
- 必要なクレート
  - chrono: 日時処理
  - clap: コマンドライン引数の解析
  - serde: シリアライズ/デシリアライズ
  - serde_json: JSONフォーマット処理

## インストール方法

1. Rustをインストールしていない場合は、[Rust公式サイト](https://www.rust-lang.org/tools/install)からインストールしてください。

2. このリポジトリをクローンします：

    ```shell
    git clone <リポジトリURL>
    cd calendar
    ```

3. (任意)初期設定として、`schedules.json`ファイルを作成します。サンプルファイルからコピーできます：

    ```shell
    cp schedules.json.example schedules.json
    ```

## 使用方法

### 予定の一覧表示

- `schedules.json`の内容を出力

```shell
cargo run -- list
```

### 予定の追加

- `schedules.json`に追記を行います
  - 予定が重複する場合は、エラーを出力します

例: 2025年5月1日の10:00から11:00までの「会議」という予定を追加します。

```shell
cargo run -- add "会議" "2025-05-01T10:00:00" "2025-05-01T11:00:00"
```

### 予定の削除

- 予定の削除を行い、結果を`schedules.json`に対し保存します

例: ID 1の予定を削除します。

```shell
cargo run -- delete 1
```

## データ形式

`schedules.json`ファイルに以下の形式でデータが保存されます：

```json
{
  "schedules": [
    {
      "id": 1,
      "subject": "予定名",
      "start": "2025-05-01T22:13:00",
      "end": "2025-05-01T22:14:00"
    }
  ]
}
```

## 開発者向け情報

### テスト実行

単体テストを実行するには、以下のコマンドを使用します：

```shell
cargo test
```

テストは主に以下の機能を検証します：

- 予定の重複チェック処理
- 予定の削除処理

### カバレッジ計測(`llvm-cov`)

```shell
cargo install cargo-llvm-cov
rustup component add llvm-tools

cargo llvm-cov
```
