FROM golang:1.23-alpine AS builder

WORKDIR /app

# Go modules
COPY go.mod go.sum ./
RUN go mod download

# ソースコードをコピー
COPY . .

# バイナリをビルド
RUN CGO_ENABLED=0 GOOS=linux go build -a -installsuffix cgo -o client .

FROM alpine:latest

# CA証明書を追加
RUN apk --no-cache add ca-certificates

WORKDIR /root/

# バイナリをコピー
COPY --from=builder /app/client .

# SPIREソケット用のディレクトリとCA証明書用のディレクトリ
RUN mkdir -p /tmp/spire-agent/public /opt/certs

CMD ["./client"]