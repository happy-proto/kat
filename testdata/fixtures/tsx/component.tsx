type CardProps = {
  title: string;
  active?: boolean;
};

export function PreviewCard({ title, active = true }: CardProps) {
  return <section className={active ? "card active" : "card"}>{title}</section>;
}
