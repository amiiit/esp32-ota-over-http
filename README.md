# ESP32 OTA over HTTPS

This project is a learning project, in which I have set up the esp32 application to fetch a certain URL in order to read what version it should run. If the local version isn't the same as this target version, it tries to download the binary, flash itself and restart.

## Next steps

1. Integrate this with a CI/CD workflow
2. Build an actual real-world appliction with this
