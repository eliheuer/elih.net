// ======================================================
//  THE LAVA LAMP BACKGROUND SHADER!
// ======================================================
// This little program does NOT run on the regular
// computer brain (the CPU) — it runs on the GRAPHICS
// CARD (the GPU), which runs it for EVERY SINGLE DOT
// on the screen, millions of times, all at once!
//
// It paints swirly rainbow blobs using only sine waves —
// the same wiggly sin() math that makes our bad guy
// bounce. Lots of waves added together = lava lamp!

#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::globals

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // "uv" is WHERE we are on the background:
    // (0,0) is one corner, (1,1) is the other.
    // Multiplying by 10 makes the pattern repeat more.
    let spot = in.uv * 10.0;

    // The clock! It ticks up forever, which is what
    // makes the pattern MOVE and swirl.
    let t = globals.time * 0.4;

    // FOUR sine waves, each wiggling a different way:
    let wave1 = sin(spot.x + t);                    // side to side
    let wave2 = sin(spot.y + t * 1.3);              // up and down
    let wave3 = sin(spot.x + spot.y + t * 1.7);     // diagonal
    let wave4 = sin(length(spot - vec2(5.0, 4.0)) * 1.4 - t * 2.0); // rings!

    // ADD the waves together and shrink back down.
    // Adding waves makes blobby, cloudy shapes.
    let blob = (wave1 + wave2 + wave3 + wave4) / 4.0;

    // Turn the blob number into LAVA LAMP colors!
    // "heat" goes from 0 (cold) to 1 (hot):
    //   cold spots  → BLACK
    //   warmer      → deep RED
    //   hot         → ORANGE
    //   hottest     → YELLOW
    let heat = blob * 0.5 + 0.5;

    // "smoothstep" is a gentle on-ramp: it slides from
    // 0 up to 1 as heat crosses between the two numbers.
    let red   = smoothstep(0.15, 0.5, heat);        // red turns on first
    let green = smoothstep(0.5, 0.85, heat) * 0.6;  // then green joins in
                                                    // (red + green = orange!)
    var color = vec3<f32>(red, green, 0.0);

    // And a few big slow PINK blobs drifting on top,
    // made from their own extra-slow wave.
    let pink_wave = 0.5 + 0.5 * sin(spot.x * 0.6 - spot.y * 0.4 + t * 0.8);
    let pinkness = smoothstep(0.8, 1.0, pink_wave) * 0.7;

    // "mix" blends two colors, like mixing paint:
    // 0 = all lava color, 1 = all pink.
    color = mix(color, vec3<f32>(1.0, 0.4, 0.7), pinkness);

    // Times 0.7 so the background stays soft and dreamy
    // and doesn't fight with the bunny for attention.
    return vec4<f32>(color * 0.7, 1.0);
}
