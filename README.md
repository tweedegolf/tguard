# TGuard

TGuard is a web-based sending and decrypting service for irmaseal-encrypted messages that is currently in development at Tweede Golf.

## Running TGuard

Tguard supports local running through a docker setup. For this you need to have both docker and docker-compose installed. The application uses a database, which can be initialized with the `./setup.sh` script included. After this, a local copy of the application can be started `docker-compose up`, and the tguard website will be localy available at http://tguard.localhost
