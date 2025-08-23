class Pmat < Formula
  desc "Zero-config AI context generation and code quality toolkit with Claude Code Agent Mode"
  homepage "https://github.com/paiml/paiml-mcp-agent-toolkit"
  url "https://github.com/paiml/paiml-mcp-agent-toolkit/archive/v2.10.0.tar.gz"
  sha256 "d8aa8ade82d3c877fd140327ee64c51d9a00d91b97c5f6195c54550ca1b8c4a0"
  license "MIT"
  head "https://github.com/paiml/paiml-mcp-agent-toolkit.git", branch: "master"

  depends_on "rust" => :build

  def install
    cd "server" do
      system "cargo", "install", "--root", prefix, "--path", ".", "--locked"
    end
  end

  test do
    assert_match "pmat 2.10.0", shell_output("#{bin}/pmat --version")
    
    # Test basic functionality
    system "#{bin}/pmat", "context", "--help"
    system "#{bin}/pmat", "agent", "--help"
    
    # Test MCP server can start (will exit quickly without stdin)
    output = shell_output("timeout 2s #{bin}/pmat agent mcp-server 2>&1 || true")
    assert_match(/MCP server|JSON-RPC|protocol/, output.downcase)
  end
end