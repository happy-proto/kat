const palette = css(`
  :root {
    --accent: #ff79c6;
  }
`);

const payload = json(`
  {
    "enabled": true
  }
`);

const markup = html(`<section data-kind="preview">${payload}</section>`);
