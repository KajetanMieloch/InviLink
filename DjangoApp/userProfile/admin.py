from django.contrib import admin
from .models import UserProfile

# Rejestracja modelu UserProfile w panelu admina
class UserProfileAdmin(admin.ModelAdmin):
    list_display = ('user', 'nickname')  # Wyświetlaj użytkownika i jego nick w panelu admina
    search_fields = ('user__username', 'nickname')  # Możliwość wyszukiwania po nazwie użytkownika i nicku

admin.site.register(UserProfile, UserProfileAdmin)
