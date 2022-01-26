# Rust Serverless Example
This example demonstrates Serverless Rust functions for AWS Lambda. This setup builds the Rust functions on the local machine, instead of the usual Docker container, to accomplish extremely fast compilation in a development environment.

This project currently depends on a few pull requests that are waiting to be merged. Until then, I have forks of those dependencies. See [dependencies](#dependencies) for details.

If anything is unclear or incorrect, don't hesitate to open an issue and I'll update this document.

## Details
Lambda Rust functions will be built using the [serverless-rust]((https://github.com/softprops/serverless-rust)) plugin, with the local building enabled for a fast development environment.

The compiled Rust functions can be ran locally (during development) using the [serverless-offline](https://github.com/dherault/serverless-offline) plugin.

The `serverless.yml` file shows a setup for an M1 Mac as development machine, deploying a Lambda function on ARM64.

## Usage
Using serverless-offline in this example depends on the AWS Lambda RIE. This emulates the Lambda Runtime Interface locally.

Get the RIE binary from the releases page of my [aws-lambda-runtime-interface-emulator fork](https://github.com/StefanNienhuis/aws-lambda-runtime-interface-emulator) ([until PR is merged](#dependencies)), and make it available in your path. 

Running locally with serverless-offline

`serverless offline --stage dev`

Creating a package

`serverless package --stage [stage]` (where stage != `dev`)

Deploying the functions

`serverless deploy --stage [stage]` (where stage != `dev`)

When stage is `dev`, the Rust functions will be compiled for your local architecture and OS. This will likely vary from the architecture and/or OS that Lambda is running on. If you do use the `dev` stage on AWS Lambda, expect `exec format error` errors.

### Manual installation
If you want to use this in an existing project or like to start from scratch. Run the following commands.

Create a project directory and enter it

`mkdir <service_name>`
`cd <service_name>`

Install serverless.

`npm install -D serverless`

Make sure my fork of the AWS RIE is available in your path if you want to use serverless-offline, according to the start of [Usage](#usage).

Install serverless-rust from GitHub ([until new NPM version is released](https://github.com/softprops/serverless-rust/issues/109)) in the parent directory. (Note: installing serverless-rust from Git with `npm install` causes a dependency issue)

`cd ../`
`git clone https://github.com/softprops/serverless-rust`

Install its dependencies.

`cd serverless-rust`
`npm install`

Return to the Serverless service root folder.

`cd ../<service_name>`

Install my serverless-offline fork ([until PRs are merged](#dependencies)).

`npm install -D @stefannienhuis/serverless-offline`

(Optionally) initialize a new Cargo workspace in the root directory.

`cargo install cargo-workspaces` (if not already installed)

`cargo workspaces init`

Remove any default members from the Cargo.toml file.

Create a new Rust package for the function

`cargo new functions/<function_name> --vcs none` (vcs=none, so it doesn't create nested Git repositories)

Add the `lambda_http` and `tokio` dependencies to the newly created package Cargo.toml similar to [functions/ping/Cargo.toml](functions/ping/Cargo.toml).

Use `serverless.yml` and the code in the functions directory as a starting point for your own functions.

Optionally add the npm scripts from `package.json` and `nodemon.json` for convenience.

## Config
```yml
service: rust-serverless-example

provider:
  name: aws
  region: eu-west-1
  runtime: rust
  architecture: arm64 # x86_64 or arm64
  memorySize: 128
  lambdaHashingVersion: '20201221'

plugins:
  - ../serverless-rust
  - ../serverless-offline

package:
  individually: true

functions:
  ping:
    handler: ping # Rust package name
    name: ${self:service}-${opt:stage}-ping

    events:
      - httpApi:
          method: 'GET'
          path: '/getping' # /ping is reserved

  hello:
    handler: hello # Rust package name
    name: ${self:service}-${opt:stage}-hello

    events:
      - httpApi:
          method: 'POST'
          path: '/hello'

custom:
  stageOptions:
    target:
      dev: aarch64-apple-darwin # Local target
      default: aarch64-unknown-linux-musl # Lambda target

    linker:
      dev: clang # Linker for local target
      default: aarch64-unknown-linux-musl-gcc # Linker for Lambda target

  serverless-offline:
    useRIE: true
    riePortRange: 6000-6499

  rust:
    dockerless: true
    target: ${self:custom.stageOptions.target.${opt:stage}, self:custom.stageOptions.target.default}
    linker: ${self:custom.stageOptions.linker.${opt:stage}, self:custom.stageOptions.linker.default}
```

## Config details

1. provider -> architecture

    Either `x86_64` or `arm64`.

2. functions -> *function* -> handler
    
    This should be your Rust package name, as specified in Cargo.toml

3. custom -> stageOptions -> target

    For `dev`, specify the target that should be used to compile for running on your local machine. You can find this by running `rustc -Vv`, under host.

    For `default`, specify the target that should be used for cross compiling for running on AWS Lambda. See [Cross Compiling](#cross-compiling) for details.

4. custom -> stageOptions -> linker

    For `dev`, specify the linker that should be used to compile for running on your local machine. For macOS, this would be `clang`. For Linux, you probably know which linker you'd like to use. For Windows, it might be `rust-lld`, but I don't know for sure.

    For `default`, specify the linker that should be used for cross compiling for running on AWS Lambda. See [Cross Compiling](#cross-compiling) for details.

## Cross Compiling

### Target
To cross compile your function for AWS Lambda, you first need to know the `target`. If you're deploying a Lambda to x86_64, this will be `x86_64-unknown-linux-musl`. If you're deploying to ARM64, this will be `aarch64-unknown-linux-musl`. See the [AWS Documentation](https://docs.aws.amazon.com/lambda/latest/dg/foundation-arch.html) for details.

Then, you need to add this target with Rustup.

`rustup target add [target]`

### Linker

The linker is dependent on the architecture that you're deploying the Lambda to. **Not** the architecture of your machine.

#### Linux
*Based on [serverless-rust docs](https://github.com/softprops/serverless-rust)*
On Linux you need to install the `musl-tools` package, or the equivalent for your distro.

```sh
$ sudo apt-get install musl-tools
```

Your linker is `musl-gcc`.

#### macOS
On macOS you need to install a MUSL cross compilation toolchain.

If you are [not running Apple Silicon](https://github.com/FiloSottile/homebrew-musl-cross/issues/23), you can install [FiloSottile/homebrew-musl-cross](https://github.com/FiloSottile/homebrew-musl-cross). For Lambda x86_64 install without parameters. For Lambda ARM64 run brew install with `--with-aarch64 --without-x86_64`. Your linker is `x86_64-linux-musl-gcc` for Lambda x86_64 or `aarch64-linux-musl-gcc` for Lambda ARM64.

If you are running Apple Silicon or are too lazy to let Homebrew compile the toolchain, you can install precompiled toolchains from [messense/homebrew-macos-cross-toolchains](https://github.com/messense/homebrew-macos-cross-toolchains). Your linker is `x86_64-unknown-linux-musl-gcc` for Lambda x86_64 or `aarch64-unknown-linux-musl-gcc` for Lambda ARM64.

#### Windows
I do not have a Windows machine at hand nor do I have the knowledge to provide these instructions. If someone gets it to work, feel free to open a PR.

## Dependencies

This project relies on a few dependencies that I forked and opened a pull request for. Below you can see the status and fork of those pull requests.

The AWS RIE (Runtime Interface Emulator) is an underlying dependency of the changes made to serverless-offline, so the pull request for serverless-offline is waiting for that. 

| Merged                                       | Repo                                                                                                      | Description                             | Pull Request                                                                | Fork                                                                                                                                             |
| -------------------------------------------- | --------------------------------------------------------------------------------------------------------- | --------------------------------------- | --------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| :heavy_check_mark:<br>(npm not released yet) | [softprop/serverless-rust](https://github.com/softprops/serverless-rust)                                  | Custom target and linker arguments      | [#112](https://github.com/softprops/serverless-rust/pull/112)               | [StefanNienhuis/serverless-rust#local-target-linker-options](https://github.com/StefanNienhuis/serverless-rust/tree/local-target-linker-options) |
| :x:                                          | [aws/aws-lambda-runtime-interface-emulator](https://github.com/aws/aws-lambda-runtime-interface-emulator) | Address argument for multiple instances | [#62](https://github.com/aws/aws-lambda-runtime-interface-emulator/pull/62) | [StefanNienhuis/aws-lambda-runtime-interface-emulator](https://github.com/StefanNienhuis/aws-lambda-runtime-interface-emulator)                  |
| :x:                                          | [dherault/serverless-offline](https://github.com/dherault/serverless-offline)                             | Implement AWS Lambda RIE Runner         | Awaiting the above                                                          | [StefanNienhuis/serverless-offline](https://github.com/StefanNienhuis/serverless-offline)                                                        |
