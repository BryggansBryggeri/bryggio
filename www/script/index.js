const app = new Vue({ 
    el: "#app",
    data: {
        name: "BRYGGANS BRYGGERI BÄRS BB",
        message: "Närstrid och torrhumling",
        counter: 0,
        recipe: "Lager",
        temp: 20.2,
    },
    created() {
			console.log("Creating brew app")
    }
});

function update(app) {
  console.log("Updating " + app.name);
  fetch("/start_measure");
}
