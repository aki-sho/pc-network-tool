# NexaLink Monitor

NexaLink Monitor は、Windows PC のネットワーク設定と通信状態を確認するためのデスクトップアプリです。アダプター、IP / DNS、TCP の通信を 1 画面で確認できます。

## ダウンロード

最新版は [GitHub Releases](https://github.com/aki-sho/nexalink-monitor/releases/latest) から取得できます。Windows x64 と WebView2 Runtime が必要です。

| ファイル | 用途 |
| --- | --- |
| `NexaLink-Monitor-Setup-…exe` | 現在の Windows ユーザー向けの NSIS インストーラー |
| `NexaLink-Monitor-Portable-…exe` | インストール不要の単体 EXE |
| `NexaLink-Monitor-Portable-…zip` | ポータブル EXE、利用説明、ライセンス、署名情報 |

`SelfSigned` と付いた配布物には自己署名のコード署名を使用しています。Windows が発行元を信頼できない旨の警告を表示することがあります。EXE と ZIP の SHA-256 は、同じ名前の `.sha256` ファイルで確認できます。

## 確認できる情報

- パブリック IP と IPv4 の既定アダプター
- ネットワークアダプターごとの状態、IPv4 / IPv6、MAC アドレス、DHCP、ゲートウェイ、DNS
- TCP の確立済み・待受中セッションの総数と、最大 100 件のアドレス、ポート、状態、PID、プロセス名

起動時と「再取得」の操作時にスナップショットを取得します。パブリック IP の取得には [ipify](https://www.ipify.org/) への HTTPS 接続を使用します。

## データの保存先

- インストール版は `%APPDATA%\NexaLink Monitor` と `%LOCALAPPDATA%\NexaLink Monitor` を使用します。
- ポータブル版は EXE と同じ場所に `NexaLink-Monitor-PortableData` を作成します。移動時は EXE とデータフォルダーを一緒に移動してください。
- 新しいデータフォルダーがなく旧版のフォルダーがある場合、`PC Network Tool` または `PC-Network-Tool-PortableData` を使用します。

## 開発

Windows 上で Node.js、npm、Rust の MSVC ツールチェーン、Visual Studio C++ Build Tools、WebView2 Runtime を用意します。

```powershell
npm ci
npm run tauri:dev
npm test
```

| フォルダー | 内容 |
| --- | --- |
| `frontend/` | 画面、アイコン、スタイル、表示ロジック |
| `src-tauri/` | Windows 側のネットワーク取得、アプリ設定、バンドル用アイコン |
| `scripts/` | Windows 配布物のビルド、署名、署名テスト |
| `tests/` | フロントエンドの回帰テスト |

ネットワークの取得には Windows 標準の PowerShell を使用します。実行先と標準モジュールの探索先を固定し、各実行に 30 秒のタイムアウトと各出力に 4 MiB の上限を設けています。パブリック IP の取得には 5 秒と 1 KiB の上限を設けています。

## Windows 向け配布ビルド

Windows の現在のユーザーの個人証明書ストアにあるコード署名証明書を指定します。署名、タイムスタンプ、SHA-256 を検証した配布物が `dist/` に作成されます。

```powershell
$env:WINDOWS_CERTIFICATE_THUMBPRINT = 'コード署名証明書のサムプリント'
npm run release:windows
```

自己署名で検証する場合は、以下の手順で `SelfSigned` 付きのファイルを生成します。

```powershell
powershell -NoProfile -File scripts/new-signing-certificate.ps1
npm run release:windows -- -SelfSigned
npm run test:signing
```

秘密鍵は Windows の証明書ストアに保存し、配布物には公開証明書を含めます。署名のタイムスタンプは既定で DigiCert のサーバーを利用します。`-TimestampUrl` で取得先を指定できます。

GitHub Actions ではバージョンタグのコミットを Windows 上でテストし、署名とハッシュを検証してから GitHub Releases に公開します。

## ライセンス

[MIT](LICENSE)
