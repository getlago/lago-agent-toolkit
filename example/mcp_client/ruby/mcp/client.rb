# frozen_string_literal: true

require "json"
require "net/http"
require "securerandom"

require_relative "config"
require_relative "tool"

PROTOCOL_VERSION = "2024-11-05"
CLIENT_NAME = "lago-mcp-client"
CLIENT_VERSION = "0.1"

module MCP
  class Client
    def initialize(config)
      @config = config
      @session_id = nil
      @tools = []
      @thread = nil
    end

    def setup!
      init_connection
      @sse_thread = Thread.new { connect_sse }
      sleep(1)
    end

    def list_tools
      response = make_request(method: "tools/list")

      tools = response.dig(:body, "result", "tools") || []
      tools.map do |tool_data|
        Tool.new(
          name: tool_data["name"],
          description: tool_data["description"],
          input_schema: tool_data["inputSchema"]
        )
      end
    end

    def call_tool(name, arguments = {})
      response = make_request(
        method: "tools/call",
        params: {
          name:,
          arguments:
        }
      )

      response[:body]["result"]
    end

    def close_session
      sse_thread.kill if sse_thread.alive?
      make_request(method: "close")
    end

    private

    attr_accessor :config, :session_id, :sse_thread

    def make_request(method:, params: {}, id: nil)
      uri = URI(config.server_url)
      http = Net::HTTP.new(uri.host, uri.port)

      request = Net::HTTP::Post.new(uri)
      request["Content-Type"] = "application/json"
      request["Accept"] = "application/json,text/event-stream"
      request["Mcp-Session-Id"] = session_id if session_id

      body = {
        jsonrpc: "2.0",
        method:,
        params:,
        id: id || SecureRandom.uuid
      }

      request.body = body.to_json
      response = http.request(request)

      parsed_body = nil
      sse_id = nil

      response.body.split("\n").each do |line|
        if line.start_with?("data: ")
          json_string = line[6..-1]
          parsed_body = JSON.parse(json_string)
        elsif line.start_with?("id: ")
          sse_id = line[4..-1]
        end
      end

      {
        status: response.code,
        headers: response.to_hash,
        body: parsed_body,
        sse_id: sse_id
      }
    rescue => e
      { error: e.message }
    end

    def init_connection
      init_response = make_request(
        method: "initialize",
        params: {
          protocolVersion: PROTOCOL_VERSION,
          capabilities: {},
          clientInfo: {
            name: CLIENT_NAME,
            version: CLIENT_VERSION
          }
        }
      )

      @session_id ||= init_response[:headers]["mcp-session-id"]&.first

      config.logger.info("Session initialized: #{session_id}")
      config.logger.info("Server info: #{init_response[:body]["result"]["serverInfo"]}")

      make_request(method: "notifications/initialized")
    end

    def connect_sse
      uri = URI(config.server_url)

      config.logger.info("Connecting to SSE stream...")

      Net::HTTP::start(uri.host, uri.port) do |http|
        request = Net::HTTP::Get.new(uri)
        request["Mcp-Session-Id"] = session_id
        request["Accept"] = "application/json,text/event-stream"
        request["Cache-Control"] = "no-cache"

        http.request(request) do |response|
          if response.code == "200"
            config.logger.info("SSE stream connected successfully")

            response.read_body do |chunk|
              chunk.split("\n").each do |line|
                if line.start_with?("data: ")
                  data = line[6..-1]
                  begin
                    config.logger.info("SSE data: #{data}")
                  rescue JSON::ParserError
                    config.logger.debug("Non-JSON SSE data: #{data}")
                  end
                elsif line.start_with?(": ")
                  config.logger.debug("SSE keepalive received: #{line}")
                end
              end
            end
          else
            config.logger.error("Failed to connect to SSE: #{response.code} #{response.message}")
          end
        end
      end
    rescue Interrupt
      config.logger.info("SSE connection interrupted by user")
    rescue => e
      config.logger.error("SSE connection error: #{e.message}")
    end
  end
end
