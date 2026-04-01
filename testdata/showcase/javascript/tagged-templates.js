const palette = css(`
  :root {
    --accent: #ff79c6;
    --surface: #282a36;
  }
`);

const payload = json(`
  {
    "enabled": true,
    "theme": "Dracula"
  }
`);

const markup = html(`
  <section data-kind="preview">${payload}</section>
`);
