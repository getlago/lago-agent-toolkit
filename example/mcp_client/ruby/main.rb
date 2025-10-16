#!/usr/bin/env ruby

require "logger"
require "tty-prompt"
require "tty-box"
require "pastel"

require_relative "mcp/client"
require_relative "model/mistral/agent"

def create_welcome_box
  pastel = Pastel.new
  box = TTY::Box.frame(
    width: 60,
    height: 8,
    align: :center,
    title: { top_left: " 🤖 Lago Billing Assistant" }
  ) do
    "Welcome to Lago Billing Assistant!\n\n" +
    "Ask questions about: \n" +
    "• Lago invoices\n" +
    "• Customer information\n" +
    "• Billing data\n\n" +
    "Type 'exit' to exit"
  end
  puts pastel.cyan(box)
end

def check_mistral_api_key
  pastel = Pastel.new

  if ENV["MISTRAL_API_KEY"].nil? || ENV["MISTRAL_API_KEY"].strip.empty?
    error_box = TTY::Box.frame(
      width: 70,
      height: 8,
      align: :left,
      title: { top_left: " ❌ Missing Configuration " },
      style: {
        fg: :red,
        border: { fg: :red }
      }
    ) do
      "MISTRAL_API_KEY environment variable is required!\n\n" +
      "Please set it before running the application:\n\n" +
      "export MISTRAL_API_KEY=your_api_key_here\n" +
      "ruby main.rb"
    end

    puts pastel.red(error_box)
    puts pastel.red("\n💡 You can get your API key from: https://console.mistral.ai/")
    exit(1)
  end
end

def check_mistral_agent_id
  pastel = Pastel.new

  if ENV["MISTRAL_AGENT_ID"].nil? || ENV["MISTRAL_AGENT_ID"].strip.empty?
    error_box = TTY::Box.frame(
      width: 70,
      height: 8,
      align: :left,
      title: { top_left: " ❌ Missing Configuration " },
      style: {
        fg: :red,
        border: { fg: :red }
      }
    ) do
      "MISTRAL_AGENT_ID environment variable is required!\n\n" +
      "Please set it before running the application:\n\n" +
      "export MISTRAL_AGENT_ID=your_agent_id_here\n" +
      "ruby main.rb"
    end

    puts pastel.red(error_box)
    puts pastel.red("\n💡 You can get your Agent ID from: https://console.mistral.ai/")
    exit(1)
  end
end

def main
  check_mistral_api_key
  check_mistral_agent_id

  logger = Logger.new($stdout)
  logger.level = Logger::WARN
  logger.formatter = proc do |severity, datetime, _progname, msg|
    "[CLIENT] #{severity} #{datetime.strftime("%H:%M:%S.%L")} - #{msg}\n"
  end

  config = MCP::Config.new(
    server_url: "http://localhost:3001/mcp",
    logger: logger,
  )

  client = MCP::Client.new(config)
  client.setup!

  mistral_agent = Model::Mistral::Agent.new(client:, logger:)
  mistral_agent.setup!

  prompt = TTY::Prompt.new
  pastel = Pastel.new

  create_welcome_box

  loop do
    user_input = prompt.ask(pastel.green("💬 Your question:")) do |q|
      q.required(false)
      q.modify(:strip)
    end

    break if user_input.nil? || user_input.downcase == "exit"
    next if user_input.empty?

    begin
      response = nil
      prompt.say(pastel.yellow("🔄 Processing your request..."))
      response = mistral_agent.chat(user_input)

      response_box = TTY::Box.frame(
        width: 80,
        title: { top_left: " 🤖 Assistant Response " },
        style: {
          fg: :bright_blue,
          border: {
            fg: :bright_blue
          }
        }
      ) do
        response
      end

      puts "\n#{response_box}\n"
    rescue => e
      error_msg = TTY::Box.frame(
        width: 60,
        title: { top_left: " ❌ Error " },
        style: {
          fg: :red,
          border: { fg: :red }
        }
      ) do
        e.message
      end
      puts error_msg
      logger.error("Chat error: #{e.message}")
    end
  end

  puts pastel.yellow("\n👋 Goodbye! Closing session...")
  client.close_session
  logger.warn("Session closed")
rescue Interrupt
  puts "\n\n👋 Chat interrupted by user. Goodbye!"
  logger.warn("Client.interrupted by user")
rescue => e
  logger.error("Client error: #{e.message}")
  logger.error(e.backtrace.join("\n"))
end

if __FILE__ == $0
  main
end
