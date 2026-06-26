# rgecko

A command line tool that makes it easier to style text, with great*er* speed!

Based on my original implementation called [ChameleonTerminal](https://github.com/Sombody101/ChameleonTerminal), which was written in Go.

## Building

### With make (still uses cargo, but less typing)

```shell
make release
```

### With cargo

```shell
cargo build --release
```

## Usage

rgecko uses tags of a similar style to [Sprectre.Console](https://spectreconsole.net/), but looser to make it easier for general CLI usage.

To use Gecko, simply use the command with text.

```bash
gecko "Hello, World!"
```

But, this doesn't print with any color.

### Changing foreground color

To use color, you need to add a markup tag. Like So:

```bash
gecko "[cyan1]This text is cyan![/]"
```

It also uses the same color names from Spectre.Console, just with a couple small differences.

Unlike Spectre.Console, Gecko doesn't read markup tags like a stack (doesn't require a trailing `[/]` to specify the end of each color segment). You can just switch colors on the fly, and the `[/]` tag has been changed to the reset color tag. It can be used anywhere in the input string, and it will reset the console to it's default.

A list of all colors can be found [here](https://spectreconsole.net/appendix/colors). Or, you can use this command to get a list of
all colors:

```bash
gecko --listc # or --listcb to see them as background colors
```

It's recommended that you still use the `[/]` tag, at least at the end of the line to prevent runaway colors from leaking into the users prompt, or the terminals next output. An even better
solution is to add `\[\033[0m\]` to the front of your `PS1`/`PSx` environment variables to ensure nothing can mess with the next input prompt.

> [!IMPORTANT]  
> Anything that is found to be a tag will be parsed. If the data inside cannot be mapped
> to a color or style, then the data is ignored, effectively making them weirdly placed comments.

### Backgrounds

Gecko also implements an option to color the background of text. All you need to do is add the keyword `on` after the first color in a markup tag, then the color of the background color. Like so:

```bash
gecko "[cyan1 on white]This should look great![/]"
```

To change only the background and leave the foreground color alone, use an underscore (`_`) then the `on` keyword with your wanted background.

```bash
gecko "[_ on white]Let's hope their default font color isn't white..."
```

### Different Colors

Don't like the preset colors and want to use your own? Well you can use both hex an RGB values!
Gecko will adjust the color to the best of it's ability based on what the users terminal supports. You can use them like this:

```bash
gecko "[#7afb42 on rgb(0,10,10)]These colors make the console look hackery[/]"
```
