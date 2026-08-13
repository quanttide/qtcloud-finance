FROM golang:1.26-alpine AS build

WORKDIR /src

ARG GOPROXY=https://goproxy.cn,direct
ENV GOPROXY=${GOPROXY}

COPY go.mod* ./
RUN go mod download

COPY . .
RUN CGO_ENABLED=0 go build -trimpath -ldflags="-s -w" -o /out/finance-server ./cmd/server

FROM alpine:3.20

RUN addgroup -S app && adduser -S -G app app
COPY --from=build /out/finance-server /usr/local/bin/finance-server

USER app
EXPOSE 8080

ENTRYPOINT ["finance-server"]
CMD []
