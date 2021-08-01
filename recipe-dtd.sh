# Replace this with your path to mozilla central
GECKO=~/dev/gecko

# dtd files
fluent-migrator --save --overwrite \
  $GECKO/layout/tools/layout-debug/ui/locale/en-US/layoutdebug.dtd \
  $GECKO/browser/locales/en-US/chrome/overrides/netError.dtd \
  $GECKO/browser/locales/en-US/chrome/browser/browser.dtd \
  $GECKO/browser/locales/en-US/chrome/browser/places/places.dtd \
  $GECKO/browser/locales/en-US/chrome/browser/translation.dtd \
  $GECKO/devtools/client/locales/en-US/performance.dtd \
  $GECKO/devtools/client/locales/en-US/sourceeditor.dtd \
  $GECKO/mobile/locales/en-US/overrides/netError.dtd \
  $GECKO/mobile/android/locales/en-US/chrome/config.dtd \
  $GECKO/dom/locales/en-US/chrome/xml/prettyprint.dtd \
  $GECKO/dom/locales/en-US/chrome/netErrorApp.dtd \
  $GECKO/dom/locales/en-US/chrome/netError.dtd \
  $GECKO/dom/locales/en-US/chrome/global.dtd \
  $GECKO/toolkit/locales/en-US/chrome/alerts/alert.dtd \
  $GECKO/toolkit/locales/en-US/chrome/global/notification.dtd \
  $GECKO/toolkit/locales/en-US/chrome/global/dialogOverlay.dtd \
  $GECKO/toolkit/locales/en-US/chrome/global/resetProfile.dtd \
  $GECKO/toolkit/locales/en-US/chrome/global/commonDialog.dtd \
  $GECKO/toolkit/locales/en-US/chrome/global/editMenuOverlay.dtd \
  $GECKO/toolkit/locales/en-US/chrome/global/tree.dtd \
  $GECKO/toolkit/locales/en-US/chrome/global/appPicker.dtd \
  $GECKO/toolkit/locales/en-US/chrome/global/textcontext.dtd \
  $GECKO/toolkit/locales/en-US/chrome/global/videocontrols.dtd \
  $GECKO/toolkit/locales/en-US/chrome/global/globalKeys.dtd \
  $GECKO/toolkit/locales/en-US/chrome/global/datetimebox.dtd \
  $GECKO/toolkit/locales/en-US/chrome/mozapps/extensions/extensions.dtd \
  $GECKO/toolkit/locales/en-US/chrome/mozapps/downloads/unknownContentType.dtd \
  $GECKO/toolkit/content/tests/chrome/rtlchrome/rtl.dtd
