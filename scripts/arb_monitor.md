# Service Monitor

Skript na monitorovanie služieb a odosielanie notifikácií do Telegramu pri problémoch.

## Súbory

- `service_monitor.py` - hlavný skript
- `service_monitor.conf` - konfiguračný súbor
- `setup_cron.sh` - pomocný skript na nastavenie cron jobu

## Nastavenie

1. **Konfigurácia Telegram bota:**
   ```bash
   # Vytvorte bota cez @BotFather v Telegrame
   # Získajte token a chat ID
   ```

2. **Upravte konfiguráciu:**
   ```bash
   nano service_monitor.conf
   ```
   
   Nastavte:
   - `token` - token vášho Telegram bota
   - `chat_id` - ID kanála/chatu kam posielať správy  
   - `names` - zoznam služieb na monitorovanie (oddelené čiarkami)
   - `min_uptime` - minimálny čas behu v sekundách (predvolene 3600 = 1h)

3. **Inštalácia závislostí:**
   ```bash
   # Ubuntu/Debian
   sudo apt install python3 python3-requests
   
   # CentOS/RHEL
   sudo yum install python3 python3-requests
   ```

4. **Nastavenie cron jobu:**
   ```bash
   ./setup_cron.sh
   ```

## Použitie

### Manuálne spustenie
```bash
./service_monitor.py
```

### Testovanie konfigurácie
```bash
# Kontrola syntaxe
python3 -m py_compile service_monitor.py

# Dry run
python3 service_monitor.py
```

### Sledovanie logov
```bash
tail -f /var/log/service_monitor.log
```

## Získanie Chat ID pre Telegram

1. **Pre osobný chat:**
   - Napíšte svojmu botovi správu
   - Otvorte: `https://api.telegram.org/bot<TOKEN>/getUpdates`
   - Nájdite `"chat":{"id":123456789}`

2. **Pre kanál:**
   - Pridajte bota do kanála ako admin
   - Chat ID začína s `-` (napr. `-1001234567890`)

3. **Pomocný skript na získanie Chat ID:**
   ```bash
   curl -s "https://api.telegram.org/bot<TOKEN>/getUpdates" | python3 -m json.tool
   ```

## Príklady notifikácií

```
🚨 Problémy so službami na server01

❌ nginx - služba nie je spustená
⚠️ postgresql - beží len 25m 30s (minimum: 1h 0m 0s)

⏰ 2025-01-15 14:30:00
```

## Troubleshooting

- **Služba nie je nájdená:** Skontrolujte `systemctl status <service>`
- **Telegram notifikácie nefungujú:** Skontrolujte token a chat ID
- **Permission denied:** Skontrolujte práva na log súbor
- **Cron nefunguje:** Skontrolujte `journalctl -u crond` alebo `/var/log/cron`