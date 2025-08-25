# frozen_string_literal: true

module MCP
  class Config
    attr_accessor :server_url, :timeout, :headers, :logger

    def initialize(server_url:, timeout: 30, headers: {}, logger: nil)
      @server_url = server_url
      @timeout = timeout
      @headers = headers
      @logger = logger
    end
  end
end
