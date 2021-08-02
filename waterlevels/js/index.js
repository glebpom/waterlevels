import("../pkg/index.js")
    .catch(console.error)
    .then(instsance => {
        window.model  = new instsance.Model(window.levels, window.maxTime);
    })