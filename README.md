# orb-ponder
This is a personal attempt to grow as a Rust programmer by writing a CPU ray tracer from first principles. It's called **orb-ponder** because the first scene everyone raytraces is a sphere, and I've been thinking about that sphere a lot.

![FZR8H-BWIAAL4CL](https://user-images.githubusercontent.com/5649419/183699033-535b7cfe-8bf1-431f-bf00-dd442d1cbc6f.jpg)![OrbPonder](https://user-images.githubusercontent.com/5649419/183699052-5543c132-74fb-4a6b-9104-1f1863e3b4ec.jpg)![ponder](https://user-images.githubusercontent.com/5649419/184169019-47dae0f2-4777-443c-b919-6b8f84420652.jpg)


I'm generally a novice at Rust, and I'm trying to gain experience using things like traits and `Vec<Box<dyn 'a + Shape>>` in a practical project. That's the first goal of this.

I'm generally a professional at realtime rasterization, which isn't raytracing, but it's sufficiently raytracing-adjacent that it's a fun stretch to figure out the parts I don't know from the parts that I do know.

This means the Rusty parts may look like student work, and the graphics parts may have a lot of unstated assumptions that make sense if you already do this for a living. But someone on Twitter asked for the code, so here it is. MIT license, so an’ it harm none, do what ye will.

I've been keeping a Twitter thread with realtime commentary and thoughts, updated as I sporadically play with this. https://twitter.com/meggsOmatic/status/1554990114081742849

## Usage

Run it from the command line. It will output a file called `test.png` with the resulting image. Image size, and samples per pixel, can be adjusted with `-w`, `-h`, and `-s`.

The contents of the scene are currently hardcoded in `main.rs`, so you'll need to edit that to add shapes or change materials. Replacing that with something externally loaded via [`serde`] seems like a good reason to learn how to use [`serde`] for the first time, so that's a high priority. That kind of learning is kind of the point. :-)

[`serde`]: https://serde.rs/
```
USAGE:
    orb-ponder.exe [OPTIONS]

OPTIONS:
    -h, --height <HEIGHT>
        --help                   Print help information
    -m, --maxdepth <MAXDEPTH>
    -s, --samples <SAMPLES>
    -V, --version                Print version information
    -w, --width <WIDTH>
```
