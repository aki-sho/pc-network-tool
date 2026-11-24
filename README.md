# PC Network Tool

> 最新バージョン 1.0.3（2026-11-14）。概要、アダプタ一覧、IP / DNSの表示に説明を追加しました。

PC Network Tool は、Windows 上で現在のネットワーク設定や接続状態を確認するためのデスクトップアプリです。  
Tauri 2、Rust、HTML、CSS、JavaScript で構成しています。

PC のネットワーク設定を行う前や、VPN 接続、仮想マシン接続、クラウド接続の確認前に、まず自分の環境がどうなっているかを把握することを目的としています。

## バージョン

v1.0.3

## 主な機能

### 1. アダプタ一覧の表示
現在の PC に存在するネットワークアダプタを一覧表示します。Windowsが返す種類や説明に基づき、接続名の下に用途を表示します。

表示例:
- イーサネット
- Wi-Fi
- ループバック
- Bluetooth ネットワーク接続 など

### 2. IPv4 / IPv6 の表示
各アダプタに割り当てられている IPv4 / IPv6 アドレスを確認できます。

### 3. ゲートウェイの確認
接続先ネットワークのデフォルトゲートウェイ情報を確認できます。

### 4. DNS の確認
各アダプタに設定されている DNS 情報を確認できます。

### 5. MAC アドレスの表示
各アダプタの MAC アドレスを表示します。

### 6. DHCP / 固定 IP の確認
IP アドレスが DHCP で取得されているか、固定 IP 設定かを確認できます。

### 7. パブリック IP の取得
現在インターネット側から見えているパブリック IP を表示します。

### 8. 接続中の基本情報の表示
現在の接続情報を一覧で表示します。

表示例:
- Protocol
- Local Address
- Local Port
- Remote Address
- Remote Port
- State
- PID
- Process

## 利用目的

このアプリは、次のような確認を行いたい場面を想定しています。

- ネットワーク設定前に現在の状態を確認したい
- VPN 接続前後の変化を確認したい
- 仮想マシン接続時にどの IP やアダプタを見ればよいか確認したい
- ローカル環境やクラウド接続前に、通信状態を整理したい
- ポート待受や接続状況をまとめて見たい

## 動作環境

- Windows
- WebView2 Runtime
- Rust（ソースからビルドする場合）
- Node.js
- npm

## 使用技術

- Tauri 2
- Rust
- HTML
- CSS
- JavaScript
- PowerShell

## インストール

依存関係をインストールします。

npm install

## 起動方法

開発用に起動します。

npm run tauri:dev

## ビルド方法

Windows向けのNSISインストーラー、単体ポータブルEXE、ZIP、各SHA-256を
`dist` に生成します。

npm run release:windows

配布ビルドは、現在のWindowsユーザーの個人証明書ストアにあるコード署名証明書と、
署名のタイムスタンプを必須とします。証明書のサムプリントを設定して実行します。

```powershell
$env:WINDOWS_CERTIFICATE_THUMBPRINT = '証明書の40桁のサムプリント'
npm run release:windows
```

ローカル検証用の自己署名証明書は、次の手順で作成・利用できます。

```powershell
powershell -NoProfile -File scripts/new-signing-certificate.ps1
npm run release:windows -- -SelfSigned
```

秘密鍵は現在ユーザーのWindows証明書ストアに非エクスポート形式で保存します。
`.signing` には公開証明書とサムプリントだけを保存し、Gitの対象から除外します。
自己署名版はファイル名に `-SelfSigned` を付け、公開証明書と署名情報を同梱します。
Windowsの信頼設定は変更しません。自己署名は公開配布用の発行元認証やSmartScreenの警告解除を保証しません。

署名時は既定で `http://timestamp.digicert.com` にダイジェストを含む署名データを送り、
タイムスタンプを取得します。EXE本体や秘密鍵は送信しません。
別のサーバーを使う場合は `-TimestampUrl` で指定できます。
署名・タイムスタンプ・指定証明書との一致を検証してからZIPとSHA-256を生成します。
既存の配布ファイルや `PortableData` は削除しません。

開発用の署名なしビルドは `npm run tauri:build` で実行できます。

## セキュリティ上の動作

- PowerShellはWindows APIで取得したシステムディレクトリの絶対パスから起動します。
- プロファイルや対話入力を無効化し、モジュール探索をWindows標準モジュールに制限します。
- PowerShellは1回30秒、標準出力・標準エラーはそれぞれ4MiBを上限とします。
- タイムアウトや出力上限超過ではプロセスを終了し、Windows Job Objectで子孫プロセスも終了します。
- 取得処理はUIスレッドから分離し、UIとRustの両方で重複取得を防ぎます。
- パブリックIP取得は接続3秒・全体5秒、応答1KiBまでに制限し、HTTPSの固定取得先から有効なIPアドレスだけを受け入れます。
- CSPで外部スクリプト、外部ページ埋め込み、不要なWebView通信を制限します。

## テスト

```powershell
npm test
```

RustテストではPowerShellの実行先・モジュール探索、タイムアウト、出力上限、
異常終了、重複取得の抑止とIP値検証を確認します。
JavaScriptテストでは連打の抑止、失敗後の再取得、エラー文字列の安全な表示を確認します。

自己署名版の配布ビルド後は `npm run test:signing` で、証明書・タイムスタンプの検証と、
署名なし・別の署名者・改ざんしたEXEの拒否も確認できます。

## 配布形式

- `PC-Network-Tool-Setup-1.0.3.exe` はNSISインストール版
- `PC-Network-Tool-Portable-1.0.3.exe` はインストール不要の単体EXE
- `PC-Network-Tool-Portable-1.0.3.zip` は単体EXEと利用説明

インストール版のデータは通常のAppDataに保存されます。ポータブル版は、
EXEと同じ場所の `PC-Network-Tool-PortableData` だけを使用します。

## 現在のバージョンで確認できる項目

- アダプタ一覧
- IPv4 / IPv6
- ゲートウェイ
- DNS
- MAC
- DHCP / 固定 IP
- パブリック IP
- 接続中の基本情報

## 補足

- パブリック IP の取得にはインターネット接続が必要です
- 環境によっては一部の項目が取得できない場合があります
- 管理者権限や Windows の設定状態により表示内容が変わる場合があります

## 今後の拡張候補

- VPN 状態の専用表示
- 仮想アダプタの識別強化
- ルーティング情報の表示
- ポートの絞り込み機能
- 接続状態の色分け表示
- スナップショット保存機能
- 接続前後の差分比較機能

## ライセンス

MIT
