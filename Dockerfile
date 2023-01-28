FROM rust:latest

# Set the working directory for the container
WORKDIR /usr/src/sithtipah

# Copy the project files into the container
COPY . .

# Build the project
RUN cargo build --release

# Run the executable
CMD ["cargo", "run"]
