<div align="center">

  <h1><code>chester</code></h1>

  <h3>
    <strong>Like Tester but with Ch</strong>
  </h3>

  <p>
    <img src="https://img.shields.io/github/actions/workflow/status/devzbysiu/chester/ci.yml?style=for-the-badge" alt="CI status badge" />
    <a href="https://codecov.io/gh/devzbysiu/chester">
      <img src="https://img.shields.io/codecov/c/github/devzbysiu/chester?style=for-the-badge" alt="Code coverage"/>
    </a>
    <img src="https://img.shields.io/crates/l/je?style=for-the-badge" alt="License"/>
  </p>

  <h3>
    <a href="#about">About</a>
    <span> | </span>
    <a href="#installation">Installation</a>
    <span> | </span>
    <a href="#configuration">Configuration</a>
    <span> | </span>
    <a href="#license">License</a>
    <span> | </span>
    <a href="#contribution">Contribution</a>
  </h3>

<sub><h4>Built with 🦀</h4></sub>

</div>

# <p id="about">About</p>

Chester is a daemon running in the background. It listens for the changes in the
project you are working on and after every change it runs the tests. The results
are exposed via REST API on local socket, so you can build your own software on
top of that.

Example use of chester: [Always On Stats](https://github.com/devzbysiu/aos) -
display code statistics using small desktop widget.

![AOT](res/aos.gif)

**Note:** Keep in mind the GIF above is a separate project which uses chester.
Head over to [AOT](https://github.com/devzbysiu/aot) for details how to install
the widget and to learn how the status is shown. Chester is designed as an API
you can build upon.

## State Changes

The next step is not triggered unless the previous one is completed with success.

```
                                    ┌──────────────────────┐
                                    │                      │
                                    │  Watch for changes   │
                                    │                      │
                                    └──────────┬───────────┘
                                               │
                                               │
                                               │Change detected
                                               │
                                    ┌──────────▼───────────┐
                                    │                      │
                                    │       Run Check      │
                                    │                      │
                                    └──────────┬───────────┘
                                               │
                                               │
                                               │Check passed
                                               │
                                    ┌──────────▼───────────┐
                                    │                      │
                                    │      Run Tests       │
                                    │                      │
                                    └──────────┬───────────┘
                                               │
                                               │
                                               │Tests passed
                                               │
                                    ┌──────────▼───────────┐
                                    │                      │
                                    │  Update Tests Index  │
                                    │                      │
                                    └──────────┬───────────┘
                                               │
                                               │
                                               │Tests changed
                                               │
                                    ┌──────────▼───────────┐
                                    │                      │
                                    │  Run Code Coverage   │
                                    │                      │
                                    └──────────────────────┘
                                    
```

## Chester API

### Check status

```bash
curl --unix-socket "/run/user/$(id -u)/chester.sock" http://chester/check/status
```

### Tests status

```bash
curl --unix-socket "/run/user/$(id -u)/chester.sock" http://chester/tests/status
```

### Coverage status

```bash
curl --unix-socket "/run/user/$(id -u)/chester.sock" http://chester/coverage/status
```

### Update repository on which tests are running

```bash
curl --unix-socket "/run/user/$(id -u)/chester.sock" \
  -XPUT -H "Content-Type: application/json" \
  -d '{"repo_root": "<new repo path here>"}' \
  http://chester/repo/root
```

## Roadmap

- [x] Check status
- [ ] Clippy status
- [x] Tests status
- [x] Code coverage
- [ ] Project quality
  - [ ] Cyclomatic Complexity
  - [ ] Halstead Metrics (Effort to maintain code, difficulty to understand code
        etc.)
  - [ ] Maintainability Index

# <p id="installation">Installation</p>

```bash
cargo install --git https://github.com/devzbysiu/chester
```

# <p id="configuration">Configuration</p>

## --- TODO ---

# <p id="license">License</p>

This project is licensed under either of

- Apache License, Version 2.0, (LICENSE-APACHE or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

At your option.

# <p id="contribution">Contribution</p>

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
