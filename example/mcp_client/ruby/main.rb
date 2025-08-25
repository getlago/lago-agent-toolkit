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

def main
  logger = Logger.new($stdout)
  logger.level = Logger::INFO
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
