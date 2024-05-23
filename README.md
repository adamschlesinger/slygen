# slygen

CLI generator for OpenAPI documents

## Installation

### Homebrew

```shell
brew install slygen
```

## Usage

To just generate an executable all in one go you just need to give it an input document:

```shell
slygen oas.yaml
```

This will output an executable CLI named `oas`. You can change the name by specifying the output:

```shell
slygen oas.yaml -o my-cli
```

Or export the rust project so you can make further changes of your own:

```shell
slygen oas.yaml -p my-cli
```

You can also feed `slygen` a spec hosted online:
```shell
slygen "https://some.website.com/swagger.json" -o web-cli
```

And even install the built executable directly to your path:
```shell
slygen "https://some.website.com/swagger.json" -o web-cli --install
```