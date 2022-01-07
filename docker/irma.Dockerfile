FROM golang:latest AS builder
ARG IRMAGO_COMMIT_HASH
ARG IRMAGO_VERSION
RUN git clone --branch ${IRMAGO_VERSION} --depth 1 https://github.com/privacybydesign/irmago /app/irmago && git -C /app/irmago checkout ${IRMAGO_COMMIT_HASH}
WORKDIR /app/irmago/irma
RUN go mod download && go mod verify && go build

FROM debian:buster
COPY --from=builder /app/irmago/irma/irma /usr/local/bin/
ENTRYPOINT ["irma"]
