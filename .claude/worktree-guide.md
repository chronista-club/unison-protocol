# Git Worktree 管理ガイド

## 命名規則

Unisonプロジェクトでは、ワークツリーの目的を明確にするため、ダブルハイフン(`--`)をdelimiterとして使用します。

### フォーマット

```
unison--{purpose}
```

### 例

- `unison--feature` - 機能開発全般
- `unison--network-layer` - ネットワーク層の実装
- `unison--cli-commands` - CLIコマンドの実装
- `unison--planning` - プランニング・仕様管理（既存）

## 現在のワークツリー

### メインリポジトリ
- **パス**: `/Users/makoto/repos/unison`
- **ブランチ**: `main`
- **用途**: メインの開発環境

### プランニング
- **パス**: `/Users/makoto/repos/unison-planning`
- **ブランチ**: `planning/spec-and-design`
- **用途**: 仕様書、アーキテクチャ設計、ロードマップ管理

### Feature開発
- **パス**: `/Users/makoto/repos/unison--feature`
- **ブランチ**: `feature/development`
- **用途**: 新機能の開発全般

## ワークフロー

### 新しいワークツリーを作成

```bash
# 1. ブランチを作成
cd /Users/makoto/repos/unison
git checkout -b feature/my-feature

# 2. ワークツリーを作成
git checkout main
git worktree add ../unison--my-feature feature/my-feature

# 3. 作業開始
cd ../unison--my-feature
```

### ワークツリーで作業

```bash
cd /Users/makoto/repos/unison--feature

# 通常のgit操作
git add .
git commit -m "feat: 新機能を追加"
git push origin feature/development
```

### mainにマージ

```bash
cd /Users/makoto/repos/unison
git checkout main
git merge feature/development
```

### ワークツリーを削除

```bash
# ブランチが不要になったら
cd /Users/makoto/repos/unison
git worktree remove ../unison--feature
git branch -d feature/development
```

## ワークツリー一覧を確認

```bash
cd /Users/makoto/repos/unison
git worktree list
```

## ベストプラクティス

1. **目的別に分離**: 大きな機能は専用のワークツリーで開発
2. **定期的にマージ**: mainとの差分が大きくならないように
3. **不要なものは削除**: 使わなくなったワークツリーは削除
4. **命名規則を守る**: `unison--{purpose}`形式を維持

## Tips

### 複数の機能を同時開発

```bash
git worktree add ../unison--network feature/network-layer
git worktree add ../unison--codegen feature/code-generator
git worktree add ../unison--cli feature/cli-tools
```

### ワークツリーの状態を確認

```bash
git worktree list --porcelain
```

### 壊れたワークツリーの修復

```bash
# ワークツリーを手動削除した場合
git worktree prune
```
