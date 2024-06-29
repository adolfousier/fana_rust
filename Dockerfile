# Use the official Rust image as a base image
FROM rust:1.53

# Set the working directory
WORKDIR /usr/src/app

# Copy the current directory contents into the container at /usr/src/app
COPY . .

# Install any needed packages specified in requirements.txt
RUN cargo build --release

# Make port configurable via Docker environment variables
ENV PORT 8080

# Make port accessible to the world outside this container
EXPOSE $PORT

# Run the binary program produced by `cargo build`
CMD ["cargo", "run", "--release"]
