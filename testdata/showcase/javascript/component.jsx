const Panel = ({ title, children }) => (
  <section className="panel">
    <h1>{title}</h1>
    <PreviewCard data-kind="hero">{children}</PreviewCard>
  </section>
);
