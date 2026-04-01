const render = (payload) => {
  console.log(JSON.stringify(payload, null, 2));
};

render({ theme: "Dracula", nested: true });
