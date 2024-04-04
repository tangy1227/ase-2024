import("./pkg").catch(console.error).then(rust_module => {
    let handle = null
    const play_button = document.getElementById("play")
    play_button.addEventListener("click", event => {
        handle = rust_module.play()
    })
    const stop_button = document.getElementById("stop")
    stop_button.addEventListener("click", event => {
        if (handle !== null) {
            handle.free()
	        handle = null
        }
    })
    const delay_slider = document.getElementById("delay")
    delay_slider.addEventListener("change", event => {
        if (handle !== null) {
            // TODO: Connect this to your filter.
        }
    })
})
