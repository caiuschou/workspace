interface CodeBlockProps {
  code: string;
  language?: string;
  title?: string;
}

export function CodeBlock({ code, language = "bash", title }: CodeBlockProps) {
  return (
    <div className="rounded-lg overflow-hidden bg-gray-950 border border-gray-800">
      {title && (
        <div className="px-4 py-2 bg-gray-900 border-b border-gray-800 text-sm text-gray-400">
          {title}
        </div>
      )}
      <pre className="p-4 overflow-x-auto">
        <code className={`language-${language} text-sm text-gray-300`}>
          {code}
        </code>
      </pre>
    </div>
  );
}

export function InlineCode({ children }: { children: React.ReactNode }) {
  return (
    <code className="px-1.5 py-0.5 rounded bg-gray-800 text-gray-300 text-sm font-mono">
      {children}
    </code>
  );
}
