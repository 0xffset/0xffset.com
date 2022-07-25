#####################
# Rust Build system #
#####################
FROM rust:slim as rust-builder
# set nightly
RUN rustup default nightly

# use the global variable
ARG RUST_APP

# create project, copy dependencies and build with default src
RUN USER=root cargo new ${RUST_APP}
WORKDIR /${RUST_APP}
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock

RUN cargo build --release

# delete src and build files, triggers layer with compiled dependencies
RUN rm -r src
RUN rm target/release/deps/${RUST_APP}*

# copy program code and rebuild
COPY src src
RUN cargo build --release

#####################################
# SASS and TailwindCSS Build System #
#####################################
FROM node:current-slim as style-builder
RUN npm i -g tailwindcss sass

# copy style files and config
COPY public public
COPY tailwind.config.js tailwind.config.js

# build sass first
RUN npx sass public/sass:public/css

#build tailwindcss files
RUN npx tailwindcss -i ./public/css/index.css -o ./public/css/index_prod.css

###########
# Runtime #
###########
FROM debian:buster-slim
# use the global variable
ARG RUST_APP

# location of the program
ARG APP=/usr/src/rust_program

RUN apt-get update && \
    rm -rf /var/lib/apt/lists/*

# create non-root user to run program
ENV APP_USER=rust_user
RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

# copy built program binary, .env file and the folder containing the webfiles
COPY --from=rust-builder /${RUST_APP}/target/release/${RUST_APP} ${APP}/program
COPY .env ${APP}/.env
COPY --from=style-builder public ${APP}/public

RUN chown -R $APP_USER:$APP_USER ${APP}
USER $APP_USER
WORKDIR ${APP}
CMD ["./program"]%