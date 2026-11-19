# rs_auth_mail_pg

 Version: 0.9.1

 Author  : 

 date    : 2026/11/19
 
 update  :

***

Rust Axum , Mail Authrize PostgreSQL Example

* PostgreSQL DB
* node22
* Linux

***
* .env

```
DATABASE_URL=postgres://root:admin@localhost/mydb
#長い文字を設定
JWT_SECRET=your-super-secret-key-change-me
GMAIL_USER=your-name
GMAIL_APP_PASSWORD=your-pass

```

***
Table: table.sql

***
* start
```
cargo build
cargo run
```

***
* test-code
```
#request

curl -X POST localhost:3000/auth/request-otp -H 'content-type: application/json' \
  -d '{"email":"a@example.com"}'

#verify
curl -X POST localhost:3000/auth/verify-otp -H 'content-type: application/json' \
 -d '{"email":"a@example.com","code":"123456"}'
```

***
### 補足

* SMTP 設定: AsyncSmtpTransport::relay はデフォルトでポート 465（暗黙的 TLS）を使用します。ポート 587（STARTTLS）を使用する場合は starttls_relay を使用してください 。

* セキュリティ: 上記の例では OTP を平文で DB に保存していますが、本番環境ではハッシュ化（例: argon2）して保存することを推奨します。

* レート制限: 同じメールアドレスに対する OTP の連続リクエストを制限するロジック（例: IP アドレスやメールアドレスごとのクールダウン）を追加することを強く推奨します。

* Gmail の制限: 個人の Gmail アカウントには 1 日あたりの送信数制限（約 500 通）があります 。本番用途では SendGrid や AWS SES などの専用サービスへの切り替えを検討してください。

* もし、認証周りの実装を自前で書かずに、既存のクレート（rok-auth や rs-auth など）を利用したい場合や、JWT の発行部分も含めた完全なコードが必要な場合、検討下さい。

***
### version
* V_0_9_1: new

***
### blog

https://zenn.dev/knaka0209/scraps/3217c80c76b2e4

