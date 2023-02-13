// alert("Is this working?");
let webSocket = new WebSocket("ws://localhost:8080/ws/");
webSocket.addEventListener("message", (e, v) => {
    if (e.data === "Reload")
        window.location.reload()
})
