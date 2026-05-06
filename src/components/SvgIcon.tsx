export default function SvgIcon({
  name,
  prefix = "icon",
  color = "#333",
  ...props
}: {
  name: string;
  prefix: string;
  color: string;
  [prop: string]: any;
}) {
  const symbolId = `#${prefix}-${name}`;

  return (
    <svg {...props} aria-hidden="true">
      <use href={symbolId} fill={color} />
    </svg>
  );
}
