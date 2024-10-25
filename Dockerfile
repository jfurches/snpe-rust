FROM rust

SHELL [ "/bin/bash", "-c" ]

# ** Download guide **
# SNPE: https://developer.qualcomm.com/software/qualcomm-neural-processing-sdk
#
# 1. Download the SDK zip file
# 2. Unzip the SDK to snpe directory
#    ```
#    mkdir snpe
#    unzip v2.26.0.240828.zip -d snpe
#    ```
# 3. Build the docker image
#    ```
#    docker build -t snpe .
#    ```

RUN set -ex \
    && echo "Installing development packages" \
    && apt-get update

RUN apt install -y clang

# Clean up packages and delete snapdragon sdk (because we can't distribute it)
RUN apt-get clean \
    && rm -rf /var/lib/apt/lists \
    && rm -rf /snpe

ENV SNPE_ROOT=/snpe

WORKDIR /workspace
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]