Hlavni funkcionality

+ 'Generate': Vygenerovani DD (directory digest = checksum soubor pro adresar) souboru pro adresar
+ 'Check': Zkontrolovani checksum pro adresar podle DD (s moznosti ulozeni noveho v pripade nalezeni chyb)
+ 'Fast refresh': Obnoveni DD souboru generovanim checksum pouze pro soubory ktere se nenachazi v DD => byly zmeneny,
  presunuty, nebo pridany nove (zmeneny => volitelne s pouzitim data upravy, velikosti souboru), (presunuty => budou
  nalezeny pomoci checksum a jejich cesta v DD bude opravena) nebo pro soubory u kterych je checksum starsi nez (
  cas/datum) bude znovu zkontrolovan. Vygenerovani logu o: nalezene presunute soubory, nove soubory, smazane soubory,...
+ 'Full refresh': Stejne jako fast refresh, ale kontroluje checksum pro vsechny soubory (Oproti sekvenci 'Fast refresh +
  Check' nemusi cist nektere soubory znovu)

Ulozeni informaci o souboru: cesta, checksum, datum upravy, datum vygenerovani checksum, velikost

Vedlejsi funkcionality:

+ Zapis do logu
+ Ignorovani adresaru / typu souboru podle specifikovaneho patternu v danem DD (stejny format jako .gitignore)
+ Konzolove UI + podpora spusteni pomoci argumentu (napr. z externiho skriptu)
+ Zpetna kompatibilita DD souboru s nekterymi ostatnimi formaty (.md5)
+ Podpora Linux i Windows file systemu
+ Paralelni pocitani checksum v pripade detekce mnoha malych souboru
+ Detekce korupce DD souboru samotneho