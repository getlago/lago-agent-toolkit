# frozen_string_literal: true

require_relative "client"
require_relative "../../mcp/run_context"

module Model
  module Mistral
    class Agent
      def initialize(logger:, client:)
        @mistral_client = Client.new(logger:)
        @mcp_context = MCP::RunContext.new(logger:, client:)
        @conversation_history = []
      end

      def setup!
        mcp_context.setup!
        self
      end

      def chat(user_message, max_tool_iterations: 2)
        @conversation_history << {
          role: "user",
          content: user_message
        }

        iterations = 0

        while iterations < max_tool_iterations

          response = mistral_client.chat_completion(
            messages: @conversation_history,
            tools: mcp_context.to_model_tools
          )

          message = response.dig("choices", 0, "message")
          return "No response received" unless message

          @conversation_history << message

          tool_calls = message["tool_calls"]

          if tool_calls.nil? || tool_calls.empty?
            return message["content"]
          end

          tool_results = mcp_context.process_tool_calls(tool_calls)

          @conversation_history.concat(tool_results)

          iterations += 1
        end
      end

      private

      attr_accessor :mcp_context, :mistral_client, :conversation_history
    end
  end
end