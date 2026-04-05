# arborist

Git worktree をタスク単位で管理する CLI ツール。

複数の作業ブランチを同時に持ち、切り替えをすばやく行うためのラッパーです。
worktree ごとにタスク名・ステータスなどのメタデータを管理できます。

## インストール

```bash
cargo install --path .
```

または GitHub Releases からプリビルドバイナリをダウンロードしてください。

## シェル統合（必須）

`arborist go` はディレクトリ移動のために `cd` コマンドを出力します。
以下のシェル関数を設定ファイルに追加してください。

**Bash / Zsh** (`~/.bashrc` または `~/.zshrc`):

```bash
arb() { eval "$(arborist go "$@")"; }
```

**Fish** (`~/.config/fish/config.fish`):

```fish
function arb
    cd (arborist go --print-path $argv)
end
```

## クイックスタート

```bash
# 新しい作業ブランチと worktree を作成
arborist new feature/login --task "ログイン機能実装"
# → ../myapp-feature-login に worktree が作成される

# worktree 一覧を表示
arborist list

# worktree に移動（シェル関数経由）
arb login

# 変更差分をまとめて確認
arborist status

# 作業完了したらステータスを更新して削除
arborist tag login --status done
arborist rm login
```

## コマンドリファレンス

### `arborist list`

全 worktree をテーブル形式で表示します。

```
arborist list [--json] [--short]
```

| フラグ | 説明 |
|--------|------|
| `--json` | JSON 配列で出力 |
| `--short` | パスのみを1行ずつ出力 |

```bash
arborist list
arborist list --json
arborist list --short
```

---

### `arborist new <branch>`

新しいブランチと worktree を同時に作成します。

```
arborist new <branch> [--task <text>] [--path <dir>] [--base <branch>]
```

| 引数 | 説明 |
|------|------|
| `<branch>` | 新規ブランチ名 |
| `--task` | タスク名（メタデータとして保存） |
| `--path` | worktree の作成先ディレクトリ（省略時は `../<repo>-<branch>`) |
| `--base` | 分岐元ブランチ（省略時は HEAD） |

```bash
arborist new feature/search
arborist new fix/auth-bug --task "ログイン失敗のバグ修正" --base main
arborist new feature/dashboard --path ~/work/dashboard
```

**デフォルトパス**: `{repo_root}/../{repo_name}-{sanitized_branch}`
例: `~/projects/myapp` で `feature/login` → `~/projects/myapp-feature-login`

---

### `arborist rm <name>`

worktree を削除します。

```
arborist rm <name> [--force]
```

| 引数 | 説明 |
|------|------|
| `<name>` | worktree 名（前方一致で解決） |
| `--force` | 未コミットの変更があっても強制削除 |

- 対話端末では削除前に確認プロンプトを表示します
- 未コミットの変更がある場合は `--force` なしでエラーになります
- 現在いる worktree は削除できません

```bash
arborist rm login
arborist rm login --force
```

---

### `arborist tag <name>`

worktree にメタデータを設定・表示します。

```
arborist tag <name> [--task <text>] [--memo <text>] [--status <active|paused|done>]
```

フラグなしで現在のメタデータを表示します。

```bash
# 表示
arborist tag login

# 更新（指定したフィールドのみ変更）
arborist tag login --task "ログイン機能実装"
arborist tag login --status done
arborist tag login --memo "後でリファクタ必要" --status paused
```

---

### `arborist status`

全 worktree の変更差分を表示します。

```
arborist status [--short]
```

```bash
arborist status
arborist status --short
```

---

### `arborist clean`

不要な worktree をまとめて削除します。

```
arborist clean [--dry-run] [--all]
```

**削除対象の条件（いずれか）:**
- ブランチが `main` / `master` にマージ済み
- メタデータの `status` が `done`

| フラグ | 説明 |
|--------|------|
| `--dry-run` | 削除対象を表示するだけ（実際には削除しない） |
| `--all` | 確認なしで全対象を一括削除 |

```bash
arborist clean --dry-run   # 何が消えるか確認
arborist clean             # チェックボックスで選択して削除
arborist clean --all       # 全対象を一括削除
```

---

### `arborist go [name]`

指定した worktree に移動します。[シェル統合](#シェル統合必須) が必要です。

```
arborist go [name]
```

- `name` を省略するとインタラクティブな選択画面が表示されます
- `name` は前方一致で解決されます

```bash
arb            # 一覧から選択
arb login      # "login" にマッチする worktree に移動
arb feat       # "feat" で始まる worktree に移動
```

---

## メタデータ

各 worktree に以下のメタデータを紐付けられます。

| フィールド | 型 | 説明 |
|------------|-----|------|
| `task` | 文字列 | 作業内容の概要 |
| `memo` | 文字列 | 自由記述メモ |
| `status` | enum | `active` / `paused` / `done` |

メタデータは `{git_common_dir}/arborist-meta.json` に保存されます。
全 worktree から共有されます。

## 名前解決

`rm` / `tag` / `go` コマンドの `<name>` 引数は前方一致で解決されます。

- **完全一致** を優先
- 一致する worktree が **0件** → エラー
- 一致する worktree が **2件以上** → エラー（候補一覧を表示）

## ライセンス

MIT
