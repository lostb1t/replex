# Use Debian slim as the base image for its small footprint and compatibility.
FROM debian:bookworm-slim

# ARG can be used to pass the target platform for multi-platform builds.
ARG TARGETPLATFORM

# Update the package lists and install OpenSSL and CA certificates. These are often required
# for secure network communications and should match the versions expected by the application.
RUN apt-get update && apt-get install -y \
  openssl \
  ca-certificates \
  && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

# Create a directory for the application.
RUN mkdir -p /app

# Copy the pre-built binary into the image. The `bin/` directory should contain binaries
# for different architectures, named according to the target platform.
COPY bin/ /bin/

# Move the correct binary for the target platform into the application directory and
# grant execution permissions.
RUN if [ "$TARGETPLATFORM" = "linux/amd64" ]; then \
  mv /bin/x86_64-unknown-linux-gnu/replex /app/replex && chmod +x /app/replex; \
  elif [ "$TARGETPLATFORM" = "linux/arm64" ]; then \
  mv /bin/aarch64-unknown-linux-gnu/replex /app/replex && chmod +x /app/replex; \
  fi

# Specify the port on which the application listens. This should match the port
# exposed by your application.
EXPOSE 80

# Set the command to run your application. Adjust the path if necessary.
CMD ["/app/replex"]
