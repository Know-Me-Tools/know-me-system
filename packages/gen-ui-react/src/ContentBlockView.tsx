// TJ-ARCH-MOB-001 compliant
import type { ContentBlock } from "./content-block";

function assertNever(x: never): never {
  throw new Error("unhandled ContentBlock variant: " + JSON.stringify(x));
}

export interface ContentBlockViewProps {
  block: ContentBlock;
}

/**
 * Renders one ContentBlock. Exhaustive over all 11 variants — adding a variant to
 * the union without a case here is a TypeScript compile error (assertNever).
 * Styling is intentionally minimal/token-driven; consuming apps theme via shadcn
 * CSS vars. This is the presentational layer (no data fetching, no invoke()).
 */
export function ContentBlockView({ block }: ContentBlockViewProps) {
  switch (block.type) {
    case "text":
      return <p className="gen-ui-text">{block.text}</p>;
    case "thinking":
      return <p className="gen-ui-thinking" data-role="thinking">{block.text}</p>;
    case "code":
      return (
        <pre className="gen-ui-code" data-lang={block.language}>
          <code>{block.code}</code>
        </pre>
      );
    case "citation":
      return (
        <blockquote className="gen-ui-citation" cite={block.source}>
          {block.quote}
        </blockquote>
      );
    case "memory":
      return (
        <div className="gen-ui-memory" data-op={block.operation}>
          <span className="gen-ui-memory-key">{block.key}</span>
          {block.value !== null ? <span className="gen-ui-memory-value">{block.value}</span> : null}
        </div>
      );
    case "toolUse":
      return (
        <div className="gen-ui-tool-use" data-tool={block.name} data-id={block.id}>
          {block.name}
        </div>
      );
    case "toolResult":
      return (
        <div className="gen-ui-tool-result" data-error={block.isError} data-tool-use-id={block.toolUseId}>
          {block.outputJson}
        </div>
      );
    case "skill":
      return (
        <div className="gen-ui-skill" data-status={block.status}>
          {block.name}
        </div>
      );
    case "artifact":
      return (
        <div className="gen-ui-artifact" data-kind={block.kind} data-id={block.id}>
          {block.content}
        </div>
      );
    case "image":
      return (
        <img
          className="gen-ui-image"
          alt=""
          src={block.url ?? (block.dataBase64 ? `data:${block.mime};base64,${block.dataBase64}` : undefined)}
        />
      );
    case "divider":
      return <hr className="gen-ui-divider" />;
    default:
      return assertNever(block);
  }
}
