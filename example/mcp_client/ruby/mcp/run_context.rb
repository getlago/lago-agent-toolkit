# frozen_string_literal: true

module MCP
  class RunContext
    def initialize(logger:, client:)
      @logger = logger
      @client = client
      @tools = []
      @tools_results = []
    end

    def setup!
      logger.info("Setting up MCP RunContext...")
      @tools = client.list_tools
      logger.info("Loaded #{tools.length} tools: #{tools.map(&:name).join(", ")}")
    end

    def to_model_tools
      @tools.map do |tool|
        { 
          type: "function",
          function: {
            name: tool.name,
            description: tool.description,
            parameters: tool.input_schema
          }
        }
      end
    end

    def process_tool_calls(tool_calls)
      results = []

      tool_calls.each do |tool_call|
        function_name = tool_call.dig("function", "name")
        arguments = JSON.parse(tool_call.dig("function", "arguments") || "{}")

        begin
          result = call_tool(function_name, arguments)
          results << {
            tool_call_id: tool_call["id"],
            role: "tool",
            content: JSON.generate(result)
          }
        rescue StandardError => e
          logger.error("Error calling tool #{function_name}: #{e.message}")
          results << {
            tool_call_id: tool_call["id"],
            role: "tool",
            content: JSON.generate({ error: e.message })
          }
        end
      end

      results
    end

    private
    
    attr_accessor :tools, :client, :logger, :tools_results

    def get_tool(name)
      tools.find { |tool| tool.name == name }
    end

    def call_tool(name, arguments = {})
      tool = get_tool(name)
      raise "Tool '#{name}' not found" unless tool

      logger.info("Calling tool: #{name} with arguments: #{arguments}")

      result = client.call_tool(name, arguments)
      #@tools_results[name] = result

      logger.info("Tool '#{name}' completed successfully")
      result
    end
  end
end