# orb-ponder
This is a personal attempt to grow as a Rust programmer by writing a CPU ray tracer from first principles. It's called `Orb Ponder` because the first scene everyone raytraces is a sphere, and I've been thinking about that sphere a lot.

![FZR8H-BWIAAL4CL](https://user-images.githubusercontent.com/5649419/183699033-535b7cfe-8bf1-431f-bf00-dd442d1cbc6f.jpg)![OrbPonder](https://user-images.githubusercontent.com/5649419/183699052-5543c132-74fb-4a6b-9104-1f1863e3b4ec.jpg)

I'm generally a novice at Rust, and I'm trying to gain experience using things like traits and `Vec<Box<dyn 'a + Shape>>` in a practical project. That's the first goal of this.

I'm generally a professional at realtime rasterization, which isn't raytracing, but it's sufficiently raytracing-adjacent that it's a fun stretch to figure out the parts I don't know from the parts that I do know.

This means the Rusty parts may look like student work, and the graphics parts may have a lot of unstated assumptions that make sense if you already do this for a living. But someone on Twitter asked for the code, so here it is. An’ it harm none, do what ye will. :-)

I've been keeping a Twitter thread with realtime commentary and thoughts, updated as I sporadically play with this. https://twitter.com/meggsOmatic/status/1554990114081742849
