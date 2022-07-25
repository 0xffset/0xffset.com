# 0xffset.com

Repo containing the source code of my website [0xffset.com](https://0xffset.com).

The website runs on an [actix-web server](https://actix.rs/) with frontend powered by [SASS](https://sass-lang.com/) and [TailwindCSS](https://tailwindcss.com/)

### Build
To build just run `docker compose build && docker compose up`

### Development
Dependencies are [Rust](https://www.rust-lang.org/tools/install), [SASS](https://sass-lang.com/install) and [TailwindCSS](https://tailwindcss.com/docs/installation)


To run the server and style compilation local run:
1. `cargo run` to start the server,
2. `sass --watch public/sass:public/css` to compile the scss 
3. `npx tailwindcss -i ./public/css/[generated_sass].css -o /public/css/[generated_sass]_prod.css` for every (exluding map files) by sass generated css file to build tailwindcss
The HTML/SCSS/JS can be live edited and doesn't require a server restart (unlike the docker container)
