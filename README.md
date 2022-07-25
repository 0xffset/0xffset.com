# 0xffset.com

Repo containing the source code of my website [0xffset.com](https://0xffset.com).

### Build
To build just run `docker compose build && docker compose up`

### Development
Dependencies are [Rust](https://www.rust-lang.org/tools/install), [SASS](https://sass-lang.com/install) and [TailwindCSS](https://tailwindcss.com/docs/installation)
To run the server and style compilation local run:
1. `cargo run` to start the server,
2. `sass --watch public/sass:public/css` to compile the scss 
3. `npx tailwindcss -i ./public/css/[generated_sass].css -o /public/css/[generated_sass]_prod.css` for every (exluding map files) by sass generated css file to build tailwindcss
