# Multi-stage build for PDFMill
FROM rust:latest as builder

WORKDIR /app

# Copy manifests first for better caching
COPY Cargo.toml ./

# Create a dummy src/main.rs to build dependencies
RUN mkdir src && echo 'fn main() {}' > src/main.rs

# Build dependencies only (this layer will be cached)
RUN cargo build --release 2>/dev/null || true

# Remove dummy files
RUN rm -rf src

# Copy actual source code
COPY src ./src

# Build the application
RUN touch src/main.rs && cargo build --release

# Runtime stage - must match builder's glibc version
FROM debian:trixie-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    # Chrome/Chromium for HTML/Markdown conversion
    chromium \
    # LibreOffice for Office documents
    libreoffice-writer \
    libreoffice-calc \
    libreoffice-impress \
    # ImageMagick for image conversion
    imagemagick \
    librsvg2-bin \
    # Additional dependencies
    fonts-liberation \
    fonts-noto-cjk \
    fonts-noto-color-emoji \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Configure ImageMagick policy to allow PDF operations
RUN sed -i 's/rights="none" pattern="PDF"/rights="read|write" pattern="PDF"/' /etc/ImageMagick-6/policy.xml 2>/dev/null || true

# Copy the binary from builder
COPY --from=builder /app/target/release/pdfmill /usr/local/bin/pdfmill

# Create non-root user
RUN useradd -m -u 1001 pdfmill && \
    mkdir -p /tmp/pdfmill && \
    chown -R pdfmill:pdfmill /app /tmp/pdfmill

# Set environment variables for engines
ENV CHROME_PATH=/usr/bin/chromium
ENV SOFFICE_PATH=/usr/bin/soffice
ENV CONVERT_PATH=/usr/bin/convert
ENV RUST_LOG=pdfmill=info
ENV PDFMILL_ADDR=0.0.0.0:3000

USER pdfmill

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Run the application
CMD ["pdfmill"]
