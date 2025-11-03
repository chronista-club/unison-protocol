# ACP技術仕様詳細

このドキュメントは、Agent Client Protocol (ACP)の技術仕様についての詳細情報を提供します。

## プロトコル概要

### 通信フロー

```
Editor (Client)                    Agent (Server)
       │                                  │
       │──────── Initialize ─────────────►│
       │                                  │
       │◄────── Capabilities ─────────────│
       │                                  │
       │──────── Task Request ────────────►│
       │                                  │
       │◄────── Progress Updates ─────────│
       │                                  │
       │◄────── Tool Requests ────────────│
       │                                  │
       │──────── Tool Results ────────────►│
       │                                  │
       │◄────── Task Complete ────────────│
```

## JSON-RPC仕様

### 基本構造

ACPはJSON-RPC 2.0をベースにしています：

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "method_name",
  "params": {
    "param1": "value1"
  }
}
```

### 主要メソッド

#### 初期化

```json
{
  "method": "initialize",
  "params": {
    "clientInfo": {
      "name": "Zed",
      "version": "0.157.0"
    },
    "capabilities": {
      "experimental": {}
    }
  }
}
```

#### タスク実行

```json
{
  "method": "agent/executeTask",
  "params": {
    "task": {
      "description": "Refactor this function",
      "context": {
        "files": ["/path/to/file.rs"],
        "selection": {
          "start": {"line": 10, "character": 0},
          "end": {"line": 20, "character": 0}
        }
      }
    }
  }
}
```

## SDK実装ガイド

### TypeScript SDK

```typescript
import { AgentClient } from '@agentclientprotocol/sdk';

const client = new AgentClient({
  command: 'node',
  args: ['agent.js', '--acp'],
  env: process.env
});

// エージェントの起動
await client.start();

// タスクの実行
const result = await client.executeTask({
  description: 'Add error handling',
  context: {
    files: ['src/main.ts']
  }
});

// クリーンアップ
await client.stop();
```

### Rust SDK

```rust
use agent_client_protocol::{Agent, Task, TaskContext};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // エージェントの作成
    let agent = Agent::new("claude-code", vec!["--acp"])?;
    
    // 初期化
    agent.initialize().await?;
    
    // タスクの実行
    let task = Task {
        description: "Optimize this function".to_string(),
        context: TaskContext {
            files: vec!["src/lib.rs".to_string()],
            ..Default::default()
        },
    };
    
    let result = agent.execute_task(task).await?;
    
    Ok(())
}
```

### Kotlin SDK

```kotlin
import acp.kotlin.AgentClient
import acp.kotlin.Task

suspend fun main() {
    // エージェントクライアントの作成
    val client = AgentClient(
        command = "node",
        args = listOf("agent.js", "--acp")
    )
    
    // 初期化
    client.initialize()
    
    // タスクの実行
    val task = Task(
        description = "Add unit tests",
        context = TaskContext(
            files = listOf("src/Main.kt")
        )
    )
    
    val result = client.executeTask(task)
    
    // クリーンアップ
    client.shutdown()
}
```

## エージェント実装

### 最小限のエージェント

TypeScriptでの基本的なエージェント実装：

```typescript
import { AgentServer } from '@agentclientprotocol/sdk';

const server = new AgentServer({
  name: 'my-agent',
  version: '1.0.0',
  
  // タスク実行ハンドラ
  async executeTask(task) {
    console.log('Task received:', task.description);
    
    // ツールの使用
    const fileContent = await this.readFile(task.context.files[0]);
    
    // 処理の実行
    const result = await processContent(fileContent);
    
    // ツールの使用
    await this.writeFile(task.context.files[0], result);
    
    return {
      success: true,
      message: 'Task completed'
    };
  }
});

// サーバーの起動
server.start();
```

### 利用可能なツール

エージェントが使用できる主なツール：

| ツール名 | 説明 | パラメータ |
|---------|------|-----------|
| `readFile` | ファイルの読み込み | `path: string` |
| `writeFile` | ファイルの書き込み | `path: string, content: string` |
| `executeCommand` | コマンドの実行 | `command: string, args: string[]` |
| `search` | コードの検索 | `query: string, scope: string[]` |
| `getContext` | コンテキストの取得 | `type: string` |

## MCPとの統合

### MCP再利用の仕様

ACPは以下のMCP仕様を再利用：

1. **リソース**: ファイルやディレクトリの表現
2. **プロンプト**: タスク記述の構造
3. **サンプリング**: LLMリクエストの形式

### ACP固有の拡張

```typescript
// ACPの独自型
interface ACPTaskContext {
  files: string[];
  selection?: Range;
  terminal?: TerminalState;
  gitStatus?: GitStatus;
}

// MCPリソースの拡張
interface ACPResource extends MCPResource {
  editorState?: EditorState;
  diagnostics?: Diagnostic[];
}
```

## セキュリティモデル

### アクセス制御

エージェントのアクセスは、エディタが仲介します：

```typescript
// エージェントの許可設定
{
  "agent_servers": {
    "my-agent": {
      "permissions": {
        "filesystem": {
          "read": ["src/**"],
          "write": ["src/**", "!src/config/**"]
        },
        "terminal": "restricted",
        "network": "none"
      }
    }
  }
}
```

### サンドボックス化

```
┌───────────────────────────────────────┐
│           Editor (Trusted)            │
│  ┌─────────────────────────────────┐  │
│  │   Permission Manager            │  │
│  └─────────────┬───────────────────┘  │
│                │                       │
│  ┌─────────────▼───────────────────┐  │
│  │   ACP Bridge                    │  │
│  └─────────────┬───────────────────┘  │
└────────────────┼───────────────────────┘
                 │ JSON-RPC/stdio
┌────────────────▼───────────────────────┐
│   Agent Process (Sandboxed)           │
│  - ファイルアクセスは仲介を経由         │
│  - ターミナルアクセスは制限付き         │
│  - ネットワークアクセスは制御可能       │
└───────────────────────────────────────┘
```

## パフォーマンス最適化

### ストリーミング応答

大きなファイルや長時間タスクには、ストリーミングを使用：

```typescript
server.executeTask(async function* (task) {
  yield { type: 'progress', percent: 0 };
  
  for (let i = 0; i < chunks.length; i++) {
    await processChunk(chunks[i]);
    yield { 
      type: 'progress', 
      percent: (i + 1) / chunks.length * 100 
    };
  }
  
  yield { type: 'complete', result: finalResult };
});
```

### キャッシング

```typescript
// コンテキストのキャッシュ
const cache = new Map<string, Context>();

async function getCachedContext(path: string): Promise<Context> {
  if (!cache.has(path)) {
    const content = await readFile(path);
    cache.set(path, parseContext(content));
  }
  return cache.get(path)!;
}
```

## エラーハンドリング

### エラーコード

| コード | 名前 | 説明 |
|-------|------|------|
| -32700 | Parse error | JSON解析エラー |
| -32600 | Invalid Request | 無効なリクエスト |
| -32601 | Method not found | メソッドが見つからない |
| -32602 | Invalid params | 無効なパラメータ |
| -32603 | Internal error | 内部エラー |
| -32000 | Server error | サーバーエラー（汎用） |

### エラーレスポンス

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32602,
    "message": "Invalid params",
    "data": {
      "field": "task.context.files",
      "reason": "must not be empty"
    }
  }
}
```

## テスト

### ユニットテスト

```typescript
import { AgentClient } from '@agentclientprotocol/sdk';
import { MockServer } from '@agentclientprotocol/testing';

describe('AgentClient', () => {
  it('should execute task successfully', async () => {
    const mockServer = new MockServer();
    const client = new AgentClient({
      server: mockServer
    });
    
    await client.start();
    
    const result = await client.executeTask({
      description: 'Test task',
      context: { files: ['test.ts'] }
    });
    
    expect(result.success).toBe(true);
  });
});
```

### 統合テスト

```bash
# ACPコンプライアンステスト
acp-test --agent ./my-agent --spec ./test-spec.json

# パフォーマンステスト
acp-bench --agent ./my-agent --workload ./workload.json
```

## デバッグ

### ログ形式

```json
{
  "timestamp": "2025-11-04T05:14:23Z",
  "level": "info",
  "message": "Task started",
  "agent": "claude-code",
  "task": {
    "id": "task-123",
    "description": "Refactor function"
  }
}
```

### デバッグツール

```bash
# ACPトラフィックの監視
acp-trace --agent claude-code

# メッセージのキャプチャ
acp-dump --output messages.jsonl

# リプレイ
acp-replay --input messages.jsonl
```

## バージョニング

### セマンティックバージョニング

ACPはSemVerに従います：

- **MAJOR**: 互換性を破る変更
- **MINOR**: 後方互換性のある機能追加
- **PATCH**: 後方互換性のあるバグ修正

### バージョンネゴシエーション

```json
{
  "method": "initialize",
  "params": {
    "protocolVersion": "1.0.0",
    "capabilities": {
      "supportedVersions": ["1.0.0", "1.1.0"]
    }
  }
}
```

## まとめ

ACPは、エディタとエージェント間の標準化された通信を可能にし、AIコーディングアシスタントのエコシステムを開放します。この技術仕様に従うことで、どのエディタでも動作する互換性のあるエージェントを実装できます。
