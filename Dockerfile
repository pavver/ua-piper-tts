# ==========================================
# 1. ЕТАП ЗБІРКИ (BUILDER)
# ==========================================
# Використовуємо рідну платформу хоста для прискорення компіляції (без QEMU)
FROM --platform=$BUILDPLATFORM rust:slim-bookworm AS builder

WORKDIR /usr/src/ua-piper-tts

# Отримуємо цільову архітектуру від Docker
ARG TARGETARCH

# Встановлюємо залежності для крос-компіляції
RUN apt-get update && apt-get install -y --no-install-recommends \
    git \
    gcc-aarch64-linux-gnu \
    libc6-dev-arm64-cross \
    gcc-x86-64-linux-gnu \
    libc6-dev-amd64-cross \
    && rm -rf /var/lib/apt/lists/*

# Налаштовуємо лінкери для крос-компіляції в Cargo
RUN mkdir -p .cargo && \
    echo '[target.aarch64-unknown-linux-gnu]' > .cargo/config.toml && \
    echo 'linker = "aarch64-linux-gnu-gcc"' >> .cargo/config.toml && \
    echo '[target.x86_64-unknown-linux-gnu]' >> .cargo/config.toml && \
    echo 'linker = "x86_64-linux-gnu-gcc"' >> .cargo/config.toml

# Копіюємо конфіги залежностей
COPY Cargo.toml Cargo.lock ./



# Створюємо тимчасовий порожній проект для кешування залежностей
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Додаємо потрібний target та кешуємо залежності відповідно до TARGETARCH
RUN if [ "$TARGETARCH" = "amd64" ]; then \
        rustup target add x86_64-unknown-linux-gnu && \
        cargo build --release --target x86_64-unknown-linux-gnu; \
    elif [ "$TARGETARCH" = "arm64" ]; then \
        rustup target add aarch64-unknown-linux-gnu && \
        cargo build --release --target aarch64-unknown-linux-gnu; \
    else \
        cargo build --release; \
    fi

# Видаляємо тимчасовий main.rs
RUN rm -rf src

# Копіюємо реальний код проекту
COPY src ./src
RUN touch src/main.rs

# Збираємо фінальний бінарник для цільової архітектури
RUN if [ "$TARGETARCH" = "amd64" ]; then \
        cargo build --release --target x86_64-unknown-linux-gnu && \
        cp target/x86_64-unknown-linux-gnu/release/ua-piper-tts target/release-binary; \
    elif [ "$TARGETARCH" = "arm64" ]; then \
        cargo build --release --target aarch64-unknown-linux-gnu && \
        cp target/aarch64-unknown-linux-gnu/release/ua-piper-tts target/release-binary; \
    else \
        cargo build --release && \
        cp target/release/ua-piper-tts target/release-binary; \
    fi

# ==========================================
# 2. ФІНАЛЬНИЙ ЕТАП (RUNTIME)
# ==========================================
FROM debian:bookworm-slim AS runtime

ARG TARGETARCH

# Встановлюємо залежності для виконання та створення користувача
RUN apt-get update && apt-get install -y --no-install-recommends \
    ffmpeg \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Створюємо некореневого користувача ttsuser з UID/GID 1000 (співпадає з radxa на хості)
RUN addgroup --gid 1000 ttsuser && \
    adduser --uid 1000 --gid 1000 --disabled-password --gecos "" ttsuser

WORKDIR /app

# Завантажуємо відповідну версію Piper залежно від архітектури
RUN if [ "$TARGETARCH" = "amd64" ]; then \
        curl -L https://github.com/rhasspy/piper/releases/download/v1.2.0/piper_amd64.tar.gz | tar -xzf - -C /app/; \
    elif [ "$TARGETARCH" = "arm64" ]; then \
        curl -L https://github.com/rhasspy/piper/releases/download/v1.2.0/piper_arm64.tar.gz | tar -xzf - -C /app/; \
    else \
        # Fallback якщо TARGETARCH не задано (наприклад, локальний звичайний build)
        ARCH=$(dpkg --print-architecture) && \
        if [ "$ARCH" = "amd64" ]; then \
            curl -L https://github.com/rhasspy/piper/releases/download/v1.2.0/piper_amd64.tar.gz | tar -xzf - -C /app/; \
        elif [ "$ARCH" = "arm64" ] || [ "$ARCH" = "aarch64" ]; then \
            curl -L https://github.com/rhasspy/piper/releases/download/v1.2.0/piper_arm64.tar.gz | tar -xzf - -C /app/; \
        else \
            echo "Непідтримувана архітектура: $ARCH" && exit 1; \
        fi \
    fi

ENV PATH="/app/piper:${PATH}"
ENV PATH="/app/piper/lib:${PATH}"
ENV LD_LIBRARY_PATH="/app/piper"

# Копіюємо згенерований бінарник та ресурси з правильними правами власника
COPY --chown=ttsuser:ttsuser --from=builder /usr/src/ua-piper-tts/target/release-binary /app/ua-piper-tts
COPY --chown=ttsuser:ttsuser config.json /app/config.json
COPY --chown=ttsuser:ttsuser models/ /app/models/
COPY --chown=ttsuser:ttsuser data/ /app/data/


# Налаштовуємо власника для всієї директорії /app
RUN chown -R ttsuser:ttsuser /app

# Запускаємо сервіс від імені некореневого користувача
USER ttsuser

EXPOSE 9000

CMD ["/app/ua-piper-tts"]
