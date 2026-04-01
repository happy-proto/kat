#!/usr/bin/env node

class ThemePreview extends BasePreview {
  #theme;

  constructor(theme) {
    super();
    this.#theme = theme;
  }

  render(options = { pretty: true }) {
    const payload = { theme: this.#theme, options };
    return /dracula/gi.test(this.#theme) ? html`<section data-kind="preview">${payload.theme}</section>` : null;
  }
}

try {
  const preview = new ThemePreview("Dracula");
  console.log(preview.render());
} catch (error) {
  console.error(error);
}
