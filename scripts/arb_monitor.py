#!/usr/bin/env python3

import subprocess
import requests
import tomllib
import logging
import sys
import os
import time
from datetime import datetime
from pathlib import Path

class ServiceMonitor:
    def __init__(self, config_path):
        self.config_path = config_path
        self.config = None
        self.logger = None
        self.load_config()
        self.setup_logging()
    
    def load_config(self):
        if not os.path.exists(self.config_path):
            raise FileNotFoundError(f"Configuration file {self.config_path} not found")
        
        with open(self.config_path, 'rb') as f:
            self.config = tomllib.load(f)
        
        required_sections = ['telegram', 'services', 'monitoring']
        for section in required_sections:
            if section not in self.config:
                raise ValueError(f"Missing section [{section}] in configuration")
        
        if not self.config['telegram']['token']:
            raise ValueError("TELEGRAM_TOKEN is not set in configuration")
        
        if not self.config['telegram']['chat_id']:
            raise ValueError("CHAT_ID is not set in configuration")
    
    def setup_logging(self):
        log_file = self.config.get('logging', {}).get('log_file', '/tmp/monitor_arb.log')
        
        os.makedirs(os.path.dirname(log_file), exist_ok=True)
        
        logging.basicConfig(
            level=logging.INFO,
            format='%(asctime)s - %(levelname)s - %(message)s',
            handlers=[
                logging.FileHandler(log_file),
                logging.StreamHandler()
            ]
        )
        self.logger = logging.getLogger(__name__)
    
    def get_service_uptime(self, service_name):
        try:
            result = subprocess.run(
                ['systemctl', 'show', service_name, '--property', 'MainPID', '--value'],
                capture_output=True,
                text=True,
                check=True
            )
            main_pid = result.stdout.strip()
            
            if not main_pid or main_pid == '0':
                return 0
            
            proc_stat_path = f'/proc/{main_pid}/stat'
            if not os.path.exists(proc_stat_path):
                return 0
            
            start_time = os.path.getctime(proc_stat_path)
            current_time = time.time()
            uptime = int(current_time - start_time)
            
            return uptime
            
        except (subprocess.CalledProcessError, FileNotFoundError, ValueError):
            return 0
    
    def is_service_active(self, service_name):
        try:
            result = subprocess.run(
                ['systemctl', 'is-active', service_name],
                capture_output=True,
                text=True
            )
            return result.returncode == 0
        except subprocess.CalledProcessError:
            return False
    
    def format_time(self, seconds):
        if seconds < 60:
            return f"{seconds}s"
        elif seconds < 3600:
            minutes = seconds // 60
            secs = seconds % 60
            return f"{minutes}m {secs}s"
        else:
            hours = seconds // 3600
            minutes = (seconds % 3600) // 60
            secs = seconds % 60
            return f"{hours}h {minutes}m {secs}s"
    
    def send_telegram_message(self, message):
        token = self.config['telegram']['token']
        chat_id = self.config['telegram']['chat_id']
        
        url = f"https://api.telegram.org/bot{token}/sendMessage"
        
        payload = {
            'chat_id': chat_id,
            'text': message,
            'parse_mode': 'HTML',
            'disable_web_page_preview': True
        }
        
        try:
            response = requests.post(url, data=payload, timeout=10)
            response.raise_for_status()
            
            result = response.json()
            if result.get('ok'):
                self.logger.info("Telegram message sent successfully")
                return True
            else:
                self.logger.error(f"Error sending Telegram message: {result.get('description')}")
                return False
                
        except requests.RequestException as e:
            self.logger.error(f"Error sending Telegram message: {e}")
            return False
    
    def check_services(self):
        self.logger.info("=== Start service monitoring ===")
        
        services_list = self.config['services']['names']
        min_uptime = int(self.config['monitoring']['min_uptime'])
        
        issues = []
        
        for service in services_list:
            if not service:
                continue
                
            # self.logger.info(f"Checking service: {service}")
            
            if not self.is_service_active(service):
                self.logger.warning(f"Service {service} is not active")
                issues.append(f"❌ <b>{service}</b> - service is not running")
                continue
            
            uptime = self.get_service_uptime(service)
            formatted_uptime = self.format_time(uptime)
            
            if uptime < min_uptime:
                min_formatted = self.format_time(min_uptime)
                self.logger.warning(f"Service {service} running too short: {formatted_uptime} (minimum: {min_formatted})")
                issues.append(f"⚠️ <b>{service}</b> - running only {formatted_uptime} (minimum: {min_formatted})")
            else:
                self.logger.info(f"Service {service} is OK - running {formatted_uptime}")
        
        if issues:
            hostname = subprocess.run(['hostname'], capture_output=True, text=True).stdout.strip()
            timestamp = datetime.now().strftime('%Y-%m-%d %H:%M:%S')
            
            message = f"🚨 <b>Service issues on {hostname}</b>\n\n"
            message += "\n".join(issues)
            message += f"\n\n⏰ {timestamp}"
            
            self.logger.info("Found service issues, sending notification")
            self.send_telegram_message(message)
        #else:
        #    self.logger.info("All services are OK")
        
        self.logger.info("=== End service monitoring ===")
        return len(issues) == 0


def main():
    script_dir = Path(__file__).parent
    config_path = script_dir / 'arb_monitor.toml'
    
    try:
        monitor = ServiceMonitor(config_path)
        success = monitor.check_services()
        sys.exit(0 if success else 1)
        
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()
