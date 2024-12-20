# Program pro chytre generovani a kontrolu checksums souboru.

Hlavni funkcionality

+ 'Generate': Vygenerovani DD (directory digest = checksum soubor pro adresar) souboru pro adresar
+ 'Check': Zkontrolovani checksum pro adresar podle DD (s moznosti ulozeni noveho v pripade nalezeni chyb)
+ 'Fast refresh': Obnoveni DD souboru generovanim checksum pouze pro soubory ktere se nenachazi v DD => byly zmeneny,
  presunuty, nebo pridany nove (zmeneny => volitelne s pouzitim data upravy, velikosti souboru), (presunuty => budou
  nalezeny pomoci checksum a jejich cesta v DD bude opravena) nebo pro soubory u kterych je checksum starsi nez (
  cas/datum) bude znovu zkontrolovan. Vygenerovani logu o: nalezene presunute soubory, nove soubory, smazane soubory,...
+ 'Full refresh': Stejne jako fast refresh, ale kontroluje checksum pro vsechny soubory

Ulozeni informaci o souboru: cesta, checksum, datum upravy, velikost souboru.

Pouziti: Program ma konzolove UI.