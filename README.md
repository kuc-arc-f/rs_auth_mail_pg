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
### version
* V_0_9_1: new

***
### blog

