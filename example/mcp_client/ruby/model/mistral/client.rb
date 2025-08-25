# frozen_string_literal

require "json"

module Model
  module Mistral
    class Client
      MISTRAL_API_URL = "https://api.mistral.ai/v1/agents/completions"

      def initialize(logger: nil, model: "mistral-large-latest")
        @logger = logger
        @model = model
        @api_key = ENV["MISTRAL_API_KEY"]
      end

      def chat_completion(messages:, tools: nil, **options)
        payload = {
          model:,
          messages:,
          **options
        }

        if tools && !tools.size.zero?
          payload[:tools] = tools
          payload[:tool_choice] = "auto"
        end

        logger.debug("Mistral API Request: #{JSON.pretty_generate(payload)}")

        uri = URI(MISTRAL_API_URL)
        http = Net::HTTP.new(uri.host, uri.port)
        http.use_ssl = true

        request = Net::HTTP::Post.new(uri)
        request["Content-Type"] = "application/json"
        request["Authorization"] = "Bearer #{@api_key}"
        request.body = JSON.generate(payload)

        begin
          response = http.request(request)
          response_body = JSON.parse(response.body)

          if response.code != "200"
            puts "DEBUG"
            puts response.body
            raise "Mistral API Error: #{response.body["error"]["message"] rescue response.body}"
          end

          response_body
        rescue JSON::ParserError => e
          raise "Invalid JSON response from Mistral API: #{e.message}"
        rescue StandardError => e
          raise "Mistral API connection error: #{e.message}"
        end
      end

      private

      attr_accessor :logger, :model, :api_key
    end
  end
end