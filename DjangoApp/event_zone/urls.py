from django.urls import path, include
from django.conf import settings
from django.conf.urls.static import static
from django.contrib.auth import views as auth_views
from . import views

app_name = 'event_zone'

urlpatterns = [
    path('', views.home, name='home'),
    path('events', views.events, name='events'),
    path('event/<str:event_id>', views.buy_ticket, name='buy_ticket'),
    path('generate_metadata', views.generate_metadata, name="generate_metadata"),
    path('tickets', views.tickets, name='tickets'),

    path('events/', views.events, name='events'),
    path('event/<str:event_id>/', views.buy_ticket, name='buy_ticket'),
    path('generate_metadata/', views.generate_metadata, name="generate_metadata"),
    path('tickets/', views.tickets, name='tickets'),

    
]
